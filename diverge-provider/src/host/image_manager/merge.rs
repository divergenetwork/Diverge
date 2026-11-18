//! Making one image of the platforms of several a caller holds.

use diverge_sdk::shared::Identity;
use sha2::{Digest as _, Sha256};

use super::manifest::{self, Member, Parsed, media_type};
use super::{Error, Meta, Store, now};
use crate::host::image_registry::Digest;
use crate::protocol::progress::Merging;
use diverge_sdk::provider::endpoints::images::merge::client::request::Part;
use diverge_sdk::provider::endpoints::images::merge::server::response::Merged;
use diverge_sdk::shared::description::Description;

/// The merge. In order:
///
/// 1. **Each platform once.** A platform named by two parts, or twice
///    in one, is `PlatformRepeated`.
/// 2. **Each source the caller's.** A digest that does not parse, or
///    that the caller owns no image by, is `NotFound`.
/// 3. **Each platform held.** A platform a part names that the
///    caller's record of its image does not hold is `PlatformNotHeld`.
/// 4. **A member per platform's image.** From an index, every entry of
///    a platform named, by the entry's own digest and size, served as
///    its manifest names itself; from an image that is not an index,
///    the image itself.
/// 5. **The index written in its one form** — see [`manifest::index`]
///    — and named by the SHA-256 of its bytes.
/// 6. A digest the caller owns already is `Exists`, and nothing
///    changes. Else room is made for the index, or it is
///    `InsufficientCapacity`; the index is written; and the caller
///    owns it, holding every platform it names. No layer is copied:
///    the record reaches the blobs its sources' records reach.
pub async fn merge(store: &Store, client_identity: &Identity, parts: &[Part], description: Option<&Description>) -> Result<Merging, Error> {
    // Rule 1.
    let mut named: Vec<&String> = Vec::new();
    for platform in parts.iter().flat_map(|part| &part.platforms) {
        if named.contains(&platform) {
            return Ok(Merging::PlatformRepeated);
        }
        named.push(platform);
    }
    let mut members = Vec::new();
    for part in parts {
        // Rule 2.
        let Some(source) = Digest::parse(&part.digest) else {
            return Ok(Merging::NotFound);
        };
        let Some(meta) = store.owned(client_identity, &source).await? else {
            return Ok(Merging::NotFound);
        };
        // Rule 3.
        if part.platforms.iter().any(|platform| !meta.platforms.contains(platform)) {
            return Ok(Merging::PlatformNotHeld);
        }
        // Rule 4.
        let Some(body) = store.read_blob(&source).await else {
            return Ok(Merging::PlatformNotHeld);
        };
        match Parsed::parse(&body) {
            Some(Parsed::Index(entries)) => {
                for entry in entries.iter().filter(|entry| part.platforms.contains(&entry.platform())) {
                    let Some(child) = store.read_blob(&entry.descriptor.digest).await else {
                        return Ok(Merging::PlatformNotHeld);
                    };
                    members.push(Member {
                        digest: entry.descriptor.digest.clone(),
                        size: entry.descriptor.size,
                        media_type: media_type(&child),
                        platform: entry.platform(),
                    });
                }
            }
            Some(Parsed::Image { .. }) => members.push(Member {
                digest: source.clone(),
                size: body.len() as u64,
                media_type: media_type(&body),
                // Its record holds exactly its one platform.
                platform: part.platforms[0].clone(),
            }),
            None => return Ok(Merging::PlatformNotHeld),
        }
    }
    // Rule 5.
    let index = manifest::index(&members);
    let Some(digest) = Digest::parse(&format!("sha256:{}", hex::encode(Sha256::digest(&index)))) else {
        return Err(Error::Registry("the index written has no digest".to_string()));
    };
    // Rule 6.
    if store.owned(client_identity, &digest).await?.is_some() {
        return Ok(Merging::Exists(digest.to_string()));
    }
    if !store.make_room(index.len() as u64).await? {
        return Ok(Merging::InsufficientCapacity);
    }
    let attempt = store.enter().await;
    let written = store.write_bytes(attempt, &digest, &index).await;
    let outcome = match written {
        Ok(true) => own(store, client_identity, &digest, description).await,
        Ok(false) => Err(Error::Registry("the index written is not its digest's".to_string())),
        Err(error) => Err(error),
    };
    store.leave(attempt).await;
    let _ = store.prune().await;
    outcome
}

/// The ownership of the index written: every platform it names, held.
async fn own(store: &Store, client_identity: &Identity, digest: &Digest, description: Option<&Description>) -> Result<Merging, Error> {
    let platforms = store.manifest_platforms(digest).await.unwrap_or_default();
    let bytes = store.image_bytes(digest, &platforms).await;
    let meta = Meta {
        description: description.cloned(),
        bytes,
        created: now(),
        containerfile: None,
        manifest_platforms: platforms.clone(),
        platforms,
        references: Vec::new(),
    };
    if !store.own(client_identity, digest, &meta).await? {
        return Ok(Merging::Exists(digest.to_string()));
    }
    Ok(Merging::Merged(Merged {
        digest: digest.to_string(),
        bytes,
    }))
}
