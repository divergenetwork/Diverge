//! Sending an image a caller holds out to a registry.

use diverge_sdk::shared::Identity;
use std::pin::Pin;
use std::sync::Arc;

use futures_util::{Stream, stream};
use tokio::sync::mpsc::{self, Sender};

use super::manifest::{Descriptor, Parsed, media_type};
use super::{Attempt, Error, Store};
use crate::host::container_deployer::name_ok;
use crate::host::image_registry::{Digest, verified};
use crate::host::registry_client::{Refused, RegistryClient, Writer};
use crate::protocol::progress::Publishing;
use diverge_sdk::provider::endpoints::images::publish::client::request::Credential;
use diverge_sdk::provider::endpoints::images::publish::server::response::{Blob, Published};
use diverge_sdk::shared::containers::request::Reference;

/// The publish as the manager hands it back: every blob the registry
/// took as it lands, then how it ended.
pub type Publish = Pin<Box<dyn Stream<Item = Result<Publishing, Error>> + Send>>;

/// What one publish needs, owned, since it runs on a task of its own.
struct Job {
    store: Arc<Store>,
    registries: Arc<RegistryClient>,
    client_identity: Identity,
    digest: String,
    reference: Reference,
    tags: Vec<String>,
    credential: Option<Credential>,
}

/// The publish started: a task of its own runs it and says what it
/// says on the stream handed back.
///
/// In order:
///
/// 1. A digest that does not parse, or that the caller owns no image
///    by, is `NotFound`.
/// 2. An image that does not hold every platform its manifest names
///    is `Incomplete`: a registry takes no manifest naming what it was
///    not given.
/// 3. A registry the configuration does not list is
///    `RegistryUnlisted`; a name that is not a repository path, or a
///    tag that is not one — see [`tag_ok`] — is `Malformed`. Nothing
///    has been asked of any registry yet.
/// 4. An attempt is entered and every blob sent is pinned for it, so
///    that a delete and its prune cannot take a blob from under the
///    upload.
/// 5. The registry is asked what it takes, and answered with the
///    credential — see [`Writer`]. The credential is held by this job
///    and its writer and by nothing else, and both end with the
///    publish.
/// 6. **Blobs, then manifests, then tags.** For each image manifest —
///    each platform's of an index, in the index's order, or the image
///    itself — its config and layers are sent, each only where the
///    registry does not hold it, each told as a `Blob`; then, for an
///    index, each platform's manifest is put under its own digest.
///    Last the image's own manifest is put under each tag, or under
///    its digest alone where no tag is named. A tag moves only here,
///    after everything it names is there.
/// 7. The reference is added to the caller's record of the image,
///    unless it is there already, and the publish is `Published`.
/// 8. The attempt leaves, whatever the end.
pub fn publish(
    store: Arc<Store>,
    registries: Arc<RegistryClient>,
    client_identity: &Identity,
    digest: &str,
    reference: &Reference,
    tags: &[String],
    credential: Option<&Credential>,
) -> Publish {
    let job = Job {
        store,
        registries,
        client_identity: client_identity.clone(),
        digest: digest.to_string(),
        reference: reference.clone(),
        tags: tags.to_vec(),
        credential: credential.cloned(),
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

/// Whether `tag` is a tag as a registry takes one: one to 128 of
/// letters, digits, `_`, `.` and `-`, the first not `.` or `-`.
pub fn tag_ok(tag: &str) -> bool {
    let valid = |byte: u8| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-');
    (1..=128).contains(&tag.len()) && tag.bytes().all(valid) && !matches!(tag.as_bytes()[0], b'.' | b'-')
}

impl Job {
    /// The whole publish; what it returns is its end.
    async fn run(&self, sender: &Sender<Result<Publishing, Error>>) -> Result<Publishing, Error> {
        // Rules 1 to 3.
        let Some(digest) = Digest::parse(&self.digest) else {
            return Ok(Publishing::NotFound);
        };
        let Some(meta) = self.store.owned(&self.client_identity, &digest).await? else {
            return Ok(Publishing::NotFound);
        };
        if meta.manifest_platforms.iter().any(|platform| !meta.platforms.contains(platform)) {
            return Ok(Publishing::Incomplete);
        }
        if !self.registries.lists(&self.reference.registry) {
            return Ok(Publishing::RegistryUnlisted);
        }
        if !name_ok(&self.reference.name) || self.tags.iter().any(|tag| !tag_ok(tag)) {
            return Ok(Publishing::Malformed);
        }
        // Rules 4 to 8.
        let attempt = self.store.enter().await;
        let outcome = self.send(&digest, attempt, sender).await;
        self.store.leave(attempt).await;
        match outcome {
            Ok(()) => {
                let reference = self.reference.clone();
                self.store
                    .amend(&self.client_identity, &digest, |record| {
                        if !record.references.contains(&reference) {
                            record.references.push(reference);
                        }
                    })
                    .await?;
                Ok(Publishing::Published(Published { digest: digest.to_string() }))
            }
            Err(Refused::Unauthorized) => Ok(Publishing::Unauthorized),
            Err(Refused::Denied) => Ok(Publishing::Denied),
            Err(Refused::Other(error)) => Err(Error::Registry(error)),
        }
    }

    /// Rules 5 and 6, under `attempt`.
    async fn send(&self, digest: &Digest, attempt: Attempt, sender: &Sender<Result<Publishing, Error>>) -> Result<(), Refused> {
        let credential = self.credential.as_ref().map(|credential| (credential.username.as_str(), credential.password.as_str()));
        let writer = self.registries.writer(&self.reference.registry, &self.reference.name, credential).await?;
        let body = self.held(attempt, digest).await?;
        match Parsed::parse(&body) {
            Some(Parsed::Index(entries)) => {
                for entry in &entries {
                    let child = self.held(attempt, &entry.descriptor.digest).await?;
                    let Some(Parsed::Image { config, layers }) = Parsed::parse(&child) else {
                        return Err(Refused::Other(format!("`{}` is not an image manifest", entry.descriptor.digest)));
                    };
                    self.blobs(&writer, attempt, &config, &layers, sender).await?;
                    writer.put_manifest(&entry.descriptor.digest.to_string(), &media_type(&child), child).await?;
                }
            }
            Some(Parsed::Image { config, layers }) => self.blobs(&writer, attempt, &config, &layers, sender).await?,
            None => return Err(Refused::Other(format!("`{digest}` is not a manifest the provider takes"))),
        }
        let media_type = media_type(&body);
        if self.tags.is_empty() {
            return writer.put_manifest(&digest.to_string(), &media_type, body).await;
        }
        for tag in &self.tags {
            writer.put_manifest(tag, &media_type, body.clone()).await?;
        }
        Ok(())
    }

    /// One image manifest's config and layers sent, each only where
    /// the registry does not hold it, each told once it has it.
    async fn blobs(&self, writer: &Writer<'_>, attempt: Attempt, config: &Descriptor, layers: &[Descriptor], sender: &Sender<Result<Publishing, Error>>) -> Result<(), Refused> {
        let mut sent: Vec<String> = Vec::new();
        for blob in std::iter::once(config).chain(layers.iter()) {
            let name = blob.digest.to_string();
            if sent.contains(&name) {
                continue;
            }
            sent.push(name.clone());
            let Some(size) = self.store.hold(attempt, &blob.digest).await else {
                return Err(Refused::Other(format!("the Store does not hold `{name}`")));
            };
            if writer.has_blob(&name).await? {
                continue;
            }
            let Some(pieces) = self.store.pieces(&blob.digest).await else {
                return Err(Refused::Other(format!("the Store does not hold `{name}`")));
            };
            // Hashed on the way out: bytes that are not the digest's
            // end the body in an error, and the registry takes nothing.
            let body = reqwest::Body::wrap_stream(verified(blob.digest.clone(), pieces));
            writer.put_blob(&name, size, body).await?;
            let _ = sender.send(Ok(Publishing::Blob(Blob { digest: name, bytes: size }))).await;
        }
        Ok(())
    }

    /// A manifest the Store holds, pinned for the attempt.
    async fn held(&self, attempt: Attempt, digest: &Digest) -> Result<Vec<u8>, Refused> {
        self.store.pin(attempt, digest).await;
        self.store.read_blob(digest).await.ok_or_else(|| Refused::Other(format!("the Store does not hold `{digest}`")))
    }
}
