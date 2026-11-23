//! Taking an image already built into a caller's Store: from a
//! registry the provider uses, or from the caller.

use diverge_sdk::shared::Identity;
use std::pin::Pin;
use std::sync::Arc;

use futures_util::{Stream, stream};
use tokio::sync::mpsc::{self, Sender};

use super::fetch::{Fetch, Planned, Repository, check, fill, plans};
use super::{Attempt, Error, Meta, Store, now};
use crate::host::container_deployer::name_ok;
use crate::host::image_registry::Digest;
use crate::host::registry_client::RegistryClient;
use crate::protocol::image_source::ImageSource;
use crate::protocol::progress::Pushing;
use diverge_sdk::provider::endpoints::images::push::server::response::{Blob, Exists, Pushed};
use diverge_sdk::shared::containers::request::Image;
use diverge_sdk::shared::containers::response::Source;

/// The push as the manager hands it back: every blob taken from the
/// caller as it lands, then how it ended.
pub type Push = Pin<Box<dyn Stream<Item = Result<Pushing, Error>> + Send>>;

/// What one push needs, owned, since it runs on a task of its own.
struct Job {
    store: Arc<Store>,
    registries: Arc<RegistryClient>,
    client_identity: Identity,
    image: Image,
    platforms: Vec<String>,
    source: ImageSource,
}

/// The push started: a task of its own runs it and says what it says
/// on the stream handed back.
///
/// In order:
///
/// 1. A digest that does not parse is `Malformed`.
/// 2. A digest the caller owns already, holding every platform named,
///    is `Exists`: nothing moves. One it owns holding fewer goes on:
///    a push adds the platforms it lacks.
/// 3. An attempt is entered; every blob it touches is pinned for it.
/// 4. **A registry the provider uses.** Every reference whose
///    registry the configuration lists and whose name is a repository
///    path is asked for the manifest by digest, in the order the image
///    lists them; the first that answers with it serves the push, and
///    the caller is not asked. Its manifest is verified and kept, and
///    planned at the platforms named — see [`plans`]: a platform the
///    image does not have is `PlatformNotFound`. The capacity is
///    checked from the sizes the manifests declare, before any layer
///    moves, trimming cache records oldest first; every blob the
///    Store lacks is fetched, verified and written byte-exact. The
///    image is owned, with `Source::Registry { reference }`. A
///    registry that served the manifest and then fails to supply the
///    image, or supplies bytes that are not their digest's, ends the
///    push in the error; a manifest the provider does not take, or a
///    blob of another size than declared, is `Malformed`.
/// 5. **Otherwise the caller.** Its manifest is fetched by
///    `OciManifest` and verified, and planned the same way; the
///    capacity is checked; every blob the Store lacks is fetched by
///    `OciBlob`, verified and written, and told as a `Blob`; then the
///    nonce check of `OciProve`, once for each platform's image,
///    whether or not a byte moved. Every pass is the image owned,
///    with `Source::Client`; a failure is `ProofFailed`. A part the
///    caller does not supply is `Unavailable`; a manifest the
///    provider does not take, bytes that are not their digest's, or a
///    blob of another size than declared, `Malformed`.
/// 6. **All, or none.** The record is written only once every
///    platform named is in the Store and proved: a push that ends any
///    other way leaves the caller holding what it held.
/// 7. The attempt leaves and the Store is pruned, whatever the end: a
///    push that ended without an ownership loses every blob it brought
///    in that nothing else reaches.
pub fn push(
    store: Arc<Store>,
    registries: Arc<RegistryClient>,
    client_identity: &Identity,
    image: &Image,
    platforms: &[String],
    source: ImageSource,
) -> Push {
    let job = Job {
        store,
        registries,
        client_identity: client_identity.clone(),
        image: image.clone(),
        platforms: platforms.to_vec(),
        source,
    };
    let (sender, receiver) = mpsc::channel(1);
    tokio::spawn(async move {
        match job.run(&sender).await {
            Ok(end) => {
                let _ = sender.send(Ok(end)).await;
            }
            Err(error) => {
                let _ = sender.send(Err(error)).await;
            }
        }
    });
    Box::pin(stream::unfold(receiver, |mut receiver| async move {
        receiver.recv().await.map(|item| (item, receiver))
    }))
}

impl Job {
    /// The whole push; what it returns is its end.
    async fn run(&self, sender: &Sender<Result<Pushing, Error>>) -> Result<Pushing, Error> {
        let Some(digest) = Digest::parse(&self.image.digest) else {
            return Ok(Pushing::Malformed);
        };
        if let Some(meta) = self.store.owned(&self.client_identity, &digest).await? {
            if self.platforms.iter().all(|platform| meta.platforms.contains(platform)) {
                return Ok(Pushing::Exists(Exists { digest: digest.to_string() }));
            }
        }
        let attempt = self.store.enter().await;
        let outcome = self.attempt(&digest, attempt, sender).await;
        self.store.leave(attempt).await;
        let _ = self.store.prune().await;
        outcome
    }

    /// Steps 4 and 5, under `attempt`.
    async fn attempt(&self, digest: &Digest, attempt: Attempt, sender: &Sender<Result<Pushing, Error>>) -> Result<Pushing, Error> {
        for reference in &self.image.references {
            if !self.registries.lists(&reference.registry) || !name_ok(&reference.name) {
                continue;
            }
            // A registry that cannot be asked does not serve the image.
            let Ok(Some(body)) = self.registries.manifest(&reference.registry, &reference.name, &digest.to_string()).await else {
                continue;
            };
            if !self.store.write_bytes(attempt, digest, &body).await? {
                return Err(Error::Registry(format!("`{}` answered a manifest that is not {digest}", reference.registry)));
            }
            let repository = Repository {
                client: &self.registries,
                host: &reference.registry,
                name: &reference.name,
            };
            let source = Source::Registry {
                reference: format!("{}/{}@{digest}", reference.registry, reference.name),
            };
            return self.from_registry(digest, attempt, &repository, source).await;
        }
        self.from_client(digest, attempt, sender).await
    }

    /// Step 4, from the registry `repository` names.
    async fn from_registry(&self, digest: &Digest, attempt: Attempt, repository: &Repository<'_>, source: Source) -> Result<Pushing, Error> {
        let planned = match plans(&self.store, attempt, repository, digest, &self.platforms).await {
            Ok(planned) => planned,
            Err(fetched) => return registry_end(fetched),
        };
        if !self.store.make_room(planned.missing(&self.store).await).await? {
            return Ok(Pushing::InsufficientCapacity);
        }
        for part in &planned.parts {
            if let Err(fetched) = fill(&self.store, attempt, repository, part, |_, _| async {}).await {
                return registry_end(fetched);
            }
        }
        self.own(digest, &planned, source).await
    }

    /// Step 5, from the caller.
    async fn from_client(&self, digest: &Digest, attempt: Attempt, sender: &Sender<Result<Pushing, Error>>) -> Result<Pushing, Error> {
        let planned = match plans(&self.store, attempt, &self.source, digest, &self.platforms).await {
            Ok(planned) => planned,
            Err(fetched) => return client_end(fetched),
        };
        if !self.store.make_room(planned.missing(&self.store).await).await? {
            return Ok(Pushing::InsufficientCapacity);
        }
        for part in &planned.parts {
            let told = fill(&self.store, attempt, &self.source, part, |blob, bytes| {
                let blob = Blob {
                    digest: blob.digest.to_string(),
                    bytes,
                };
                async move {
                    let _ = sender.send(Ok(Pushing::Blob(blob))).await;
                }
            })
            .await;
            if let Err(fetched) = told {
                return client_end(fetched);
            }
        }
        // One proof per platform's image: the caller holds each it
        // gives, or none of them is owned.
        for part in &planned.parts {
            if !check(&self.store, &self.source, part).await? {
                return Ok(Pushing::ProofFailed);
            }
        }
        self.own(digest, &planned, Source::Client).await
    }

    /// The ownership recorded, or widened: the platforms named are
    /// held from now on, beside whatever the caller held of the digest
    /// already, in the manifest's order. `Pushed` either way.
    async fn own(&self, digest: &Digest, planned: &Planned, source: Source) -> Result<Pushing, Error> {
        let held = |already: &[String]| -> Vec<String> {
            planned
                .named
                .iter()
                .filter(|platform| already.contains(platform) || self.platforms.contains(platform))
                .cloned()
                .collect()
        };
        let platforms = held(&[]);
        let meta = Meta {
            bytes: self.store.image_bytes(digest, &platforms).await,
            created: now(),
            containerfile: None,
            manifest_platforms: planned.named.clone(),
            platforms,
            references: Vec::new(),
        };
        let bytes = if self.store.own(&self.client_identity, digest, &meta).await? {
            meta.bytes
        } else {
            // The caller owned it, holding other platforms: what it
            // holds grows, and nothing else of its record changes.
            // The platforms are widened from the record as it is
            // under the records' lock, so that two pushes of the one
            // digest ending together each keep the other's; the bytes
            // only grow, so the larger count stands whichever writes
            // last.
            let amended = self.store.amend(&self.client_identity, digest, |record| record.platforms = held(&record.platforms)).await?;
            let wider = amended.map_or_else(|| meta.platforms.clone(), |record| record.platforms);
            let bytes = self.store.image_bytes(digest, &wider).await;
            let counted = self.store.amend(&self.client_identity, digest, |record| record.bytes = record.bytes.max(bytes)).await?;
            counted.map_or(bytes, |record| record.bytes)
        };
        Ok(Pushing::Pushed(Pushed {
            digest: digest.to_string(),
            bytes,
            source,
        }))
    }
}

/// A registry's fetch that failed, as the push's end: a manifest the
/// provider does not take, or a blob of another size than declared,
/// is `Malformed`; anything else — a part the registry no longer
/// supplies, bytes that are not their digest's, a registry that could
/// not be asked — is the error, since the registry answered that it
/// serves the image.
fn registry_end(fetched: Fetch) -> Result<Pushing, Error> {
    match fetched {
        Fetch::Malformed => Ok(Pushing::Malformed),
        Fetch::Unavailable => Err(Error::Registry("a part of the image is not served".to_string())),
        Fetch::Corrupt => Err(Error::Registry("bytes that are not their digest's".to_string())),
        Fetch::Supplier(error) => Err(Error::Registry(error)),
        Fetch::Store(error) => Err(error),
        Fetch::PlatformNotFound => Ok(Pushing::PlatformNotFound),
    }
}

/// The caller's fetch that failed, as the push's end.
fn client_end(fetched: Fetch) -> Result<Pushing, Error> {
    match fetched {
        Fetch::Unavailable | Fetch::Supplier(_) => Ok(Pushing::Unavailable),
        Fetch::Malformed | Fetch::Corrupt => Ok(Pushing::Malformed),
        Fetch::Store(error) => Err(error),
        Fetch::PlatformNotFound => Ok(Pushing::PlatformNotFound),
    }
}

