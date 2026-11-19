//! Images brought onto one provider and made one image there: what a
//! merge is, and what a publish does before it uploads.

use std::sync::Arc;

use diverge_sdk::provider::endpoints::images::list::server::response::Image as Listed;
use diverge_sdk::provider::endpoints::images::merge::client::execute as merge;
use diverge_sdk::provider::endpoints::images::merge::client::request::{Frame as Merge, Part};
use diverge_sdk::shared::Identity;
use diverge_sdk::shared::containers::request::Image;
use diverge_sdk::shared::description::Description;
use diverge_sdk::shared::error::Error;
use diverge_sdk::wire::client::handle::Handle;
use futures_util::future::join_all;
use tokio::sync::mpsc;

use super::bring::{self, Brought, Failure, Step};
use crate::containers::images;
use crate::daemon::Daemon;
use crate::serve::reply;

/// One image of a request, its platforms known.
pub struct Taken {
    /// The image, by its digest.
    pub digest: String,
    /// The platforms taken of it.
    pub platforms: Vec<String>,
    /// Whether they are every platform its manifest names. Known for
    /// the one image of a request of one, where it decides whether
    /// anything is merged; not asked for otherwise, and `false`.
    pub whole: bool,
}

/// What the daemon's Store on every connected provider holds, as the
/// daemon mirrors it now: each provider that keeps one, with its
/// images. Read at once, and not waited for: a listing still arriving
/// is what has arrived.
pub async fn stores(daemon: &Daemon) -> Vec<(Identity, Vec<Listed>)> {
    let connected = daemon.live.connected_providers().await;
    let read = join_all(connected.into_iter().map(|identity| async move {
        let mirror = daemon.live.store(&identity).await?;
        Some((identity, mirror.images().await))
    }))
    .await;
    read.into_iter().flatten().collect()
}

/// The image `digest` with its platforms known — those `named` for
/// it, or every platform its manifest names — or `None` where they
/// cannot be. The rules:
///
/// 1. **What the manifest names is asked only where it is needed**:
///    where no platform is named, and where it is the request's one
///    image, `alone`, whose being whole decides whether anything is
///    merged.
/// 2. **From a Store first.** A Store the daemon mirrors that holds
///    the digest says what its manifest names, and nobody is asked.
/// 3. **Else through a source.** The first source that can supply the
///    image, of any platform, is read for the manifest, as
///    [`images::platforms`] reads it, and let go. No source, or a
///    manifest that states no platform, is `None`.
pub async fn taken(daemon: &Arc<Daemon>, held: &[(Identity, Vec<Listed>)], digest: &str, named: Option<&Vec<String>>, alone: bool) -> Option<Taken> {
    // Rule 1.
    if let (Some(platforms), false) = (named, alone) {
        return Some(Taken {
            digest: digest.to_string(),
            platforms: platforms.clone(),
            whole: false,
        });
    }
    // Rule 2.
    let listed = held.iter().flat_map(|(_, images)| images).find(|image| image.digest == digest && !image.manifest_platforms.is_empty());
    let manifest = match listed {
        Some(image) => image.manifest_platforms.clone(),
        // Rule 3.
        None => {
            let image = Image {
                digest: digest.to_string(),
                references: Vec::new(),
            };
            let source = images::locate(daemon, &image, images::Reach::Full { except: None }, &[], images::Wanted::Any).await?;
            let manifest = images::platforms(&source, digest).await;
            images::let_go(source);
            manifest?
        }
    };
    let platforms = named.cloned().unwrap_or_else(|| manifest.clone());
    let whole = manifest.iter().all(|platform| platforms.contains(platform));
    Some(Taken {
        digest: digest.to_string(),
        platforms,
        whole,
    })
}

/// Whether two of `parts` take the same platform: one image holds a
/// platform once.
pub fn repeated(parts: &[Taken]) -> bool {
    let mut seen: Vec<&String> = Vec::new();
    for platform in parts.iter().flat_map(|part| &part.platforms) {
        if seen.contains(&platform) {
            return true;
        }
        seen.push(platform);
    }
    false
}

/// The one image `parts` are, on the provider.
pub struct Joined {
    /// Its manifest digest: the merged image's, or the one part's own
    /// where nothing was merged.
    pub digest: String,
    /// Whether the request made it there: an image merged by it, or,
    /// where nothing was merged, one the daemon did not own there when
    /// the request began.
    pub created: bool,
    /// Whether it was merged, and then its size where the merge made
    /// it: `None`, nothing was merged — rule 3 of [`join`];
    /// `Some(None)`, the merge found it there already.
    pub merged: Option<Option<u64>>,
}

/// Why `parts` were not made one image.
pub enum Stopped {
    /// A part was not brought in.
    Brought(Failure),
    /// The provider says two parts take the same platform.
    PlatformRepeated,
    /// The provider's Store has no room for the image.
    InsufficientCapacity,
    /// The provider keeps no Store.
    Unsupported,
    /// A failure, in the daemon's words or the provider's.
    Error(Error),
}

/// `parts` brought onto `provider`, whose Store of the daemon's holds
/// `held` now, and made one image there, kept as `description`. Every
/// blob pulled and every set of platforms that lands is told on
/// `steps`, which is let go once nothing more is to be told. The
/// rules:
///
/// 1. **Brought in**, every part at once: the platforms of a part the
///    Store does not hold there are brought in as [`bring::bring`]
///    brings them; a part the Store holds whole is not asked about.
///    The first failure to end is the answer, once every part has
///    ended.
/// 2. **Merged**, where there is more than one part or the one part
///    is not whole: the provider's merge.
/// 3. **One whole part is itself**: nothing is merged, and the image
///    is the part, by its own digest. Only then is a part kept as the
///    description while it is brought in: what is merged is kept as
///    the description whole, not part by part.
pub async fn join(daemon: &Arc<Daemon>, provider: &Identity, handle: &Handle, held: &[(Identity, Vec<Listed>)], parts: &[Taken], description: Option<&Description>, steps: mpsc::UnboundedSender<Step>) -> Result<Joined, Stopped> {
    let there = |digest: &str| held.iter().find(|(identity, _)| identity == provider).and_then(|(_, images)| images.iter().find(|image| image.digest == digest));
    // Rule 3, decided before anything moves.
    let merges = !(parts.len() == 1 && parts[0].whole);
    let owned = !merges && there(&parts[0].digest).is_some();
    // Rule 1.
    let brought = join_all(parts.iter().map(|part| {
        let lacking: Vec<String> = match there(&part.digest) {
            Some(image) => part.platforms.iter().filter(|platform| !image.platforms.contains(platform)).cloned().collect(),
            None => part.platforms.clone(),
        };
        let steps = &steps;
        let description = if merges { None } else { description };
        async move {
            if lacking.is_empty() {
                return Brought::Exists;
            }
            let image = Image {
                digest: part.digest.clone(),
                references: Vec::new(),
            };
            bring::bring(daemon, handle, &image, description, &lacking, steps).await
        }
    }))
    .await;
    drop(steps);
    for ended in brought {
        if let Brought::Failed(failure) = ended {
            return Err(Stopped::Brought(failure));
        }
    }
    // Rule 3.
    if !merges {
        return Ok(Joined {
            digest: parts[0].digest.clone(),
            created: !owned,
            merged: None,
        });
    }
    // Rule 2.
    let ask = Merge {
        images: parts
            .iter()
            .map(|part| Part {
                digest: part.digest.clone(),
                platforms: part.platforms.clone(),
            })
            .collect(),
        description: description.cloned(),
    };
    match merge::execute(handle, &ask).await {
        Ok(merge::Outcome::Merged(merged)) => Ok(Joined {
            digest: merged.digest,
            created: true,
            merged: Some(Some(merged.bytes)),
        }),
        Ok(merge::Outcome::Exists(digest)) => Ok(Joined {
            digest,
            created: false,
            merged: Some(None),
        }),
        Err(merge::ExecuteError::PlatformRepeated) => Err(Stopped::PlatformRepeated),
        Err(merge::ExecuteError::InsufficientCapacity) => Err(Stopped::InsufficientCapacity),
        Err(merge::ExecuteError::Unsupported) => Err(Stopped::Unsupported),
        Err(merge::ExecuteError::Provider(error)) => Err(Stopped::Error(error)),
        Err(error) => Err(Stopped::Error(reply::failure(&error))),
    }
}
