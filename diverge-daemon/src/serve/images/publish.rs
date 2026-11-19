//! Publishing images to a registry as one image.

use std::sync::Arc;

use diverge_sdk::daemon::endpoints::images::publish::client::request;
use diverge_sdk::daemon::endpoints::images::publish::server::response::{Frame, Merged, Published, Uploaded};
use diverge_sdk::daemon::grant::images::Action;
use diverge_sdk::provider::endpoints::images::list::server::response::Image as Listed;
use diverge_sdk::provider::endpoints::images::publish::client::execute as publish;
use diverge_sdk::provider::endpoints::images::publish::client::request::{Credential, Frame as Publish};
use diverge_sdk::provider::endpoints::images::publish::server::response::Frame as Sent;
use diverge_sdk::shared::Identity;
use diverge_sdk::wire::client::handle::Handle;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;
use futures_util::StreamExt as _;
use futures_util::future::join_all;
use tokio::sync::mpsc;

use super::bring::{Failure, Step};
use super::join::{self, Stopped, Taken};
use crate::daemon::Daemon;
use crate::judge::{self, Standing, Who};
use crate::serve::reply;
use crate::store;
use crate::volumes;

/// Answer the publish — what happened as it happened, then how it
/// ended — and finish the scope.
pub async fn handle(scope: ScopeHandle, frame: request::Frame, who: Who, daemon: &Arc<Daemon>) {
    let answer = match serve(&scope, frame, who, daemon).await {
        Ok(answer) => answer,
        Err(error) => Frame::Error(reply::failure(&error)),
    };
    reply::reply(&scope, &answer).await;
    scope.send_response_finish().await;
}

/// The answer that ends the publish, everything that happened before
/// it sent on the scope as it happened. The rules:
///
/// 1. `Forbidden` with no `publish` grant over images at all.
/// 2. **Who may do it**, by [`candidates`]: the provider named, or the
///    answer saying why not; with none named, every provider the
///    grant reaches that can, or `NoProvider`.
/// 3. **The platforms of each part**, by [`taken`], every part at
///    once: those it names, or every platform its manifest names.
///    `Unavailable` for a digest whose manifest no source supplies.
/// 4. `PlatformRepeated` for a platform two parts take.
/// 5. **The provider**, by [`most`]: of the candidates, the one whose
///    Store of the daemon's holds the most of the platforms taken
///    now. `Error` where it is no longer connected.
/// 6. **Brought in**, every part at once: the platforms of a part the
///    Store does not hold there are brought in as
///    [`bring::bring`] brings them; a part the Store holds whole is
///    not asked about. The first failure to end ends the publish, once
///    every part has ended.
/// 7. **Merged**, where there is more than one part or the one part
///    is not whole: the provider's merge, told as `Merged`. One whole
///    part is published by its own digest, and nothing is merged.
/// 8. **Published**: the provider's publish, each blob the registry
///    takes told as `Uploaded`, and its end the answer.
/// 9. **Made, or there already.** The image published is made by the
///    request where it was merged by it, and, where nothing was
///    merged, where the daemon owned no image of the digest there
///    when the provider was picked. Only an image made is kept as the
///    request's description: one there already keeps its own.
async fn serve(scope: &ScopeHandle, frame: request::Frame, who: Who, daemon: &Arc<Daemon>) -> Result<Frame, store::Error> {
    let standing = {
        let mut conn = daemon.store.acquire().await?;
        Standing::of(&mut conn, who).await?
    };
    // Rule 1.
    let Some(standing) = standing else {
        return Ok(Frame::Forbidden);
    };
    if !judge::images::holds(&standing, Action::Publish) {
        return Ok(Frame::Forbidden);
    }
    // Rule 2.
    let able = match candidates(daemon, &standing, frame.provider.as_ref(), &frame.reference.registry).await {
        Ok(able) => able,
        Err(answer) => return Ok(answer),
    };
    scope.allowed().await;
    // Rule 3. What every connected provider's Store holds for the
    // daemon, read once: what a manifest names where a Store has the
    // digest, and what each candidate holds already.
    let held = join::stores(daemon).await;
    let alone = frame.images.len() == 1;
    let resolved = join_all(frame.images.iter().map(|part| join::taken(daemon, &held, &part.digest, part.platforms.as_ref(), alone))).await;
    let Some(parts) = resolved.into_iter().collect::<Option<Vec<Taken>>>() else {
        return Ok(Frame::Unavailable);
    };
    // Rule 4.
    if join::repeated(&parts) {
        return Ok(Frame::PlatformRepeated);
    }
    // Rule 5.
    let provider = most(&able, &held, &parts);
    let handle = match volumes::handle(daemon, &provider).await {
        Ok(handle) => handle,
        Err(fail) => return Ok(Frame::Error(fail.error())),
    };
    // Rules 6, 7 and 9, by `join::join`.
    let (steps, said) = mpsc::unbounded_channel();
    let (joined, ()) = tokio::join!(join::join(daemon, &provider, &handle, &held, &parts, frame.description.as_ref(), steps), tell(scope, said));
    let joined = match joined {
        Ok(joined) => joined,
        Err(Stopped::Brought(failure)) => return Ok(refusal(failure)),
        Err(Stopped::PlatformRepeated) => return Ok(Frame::PlatformRepeated),
        Err(Stopped::InsufficientCapacity) => return Ok(Frame::InsufficientCapacity),
        Err(Stopped::Unsupported) => return Ok(Frame::Unsupported),
        Err(Stopped::Error(error)) => return Ok(Frame::Error(error)),
    };
    if let Some(bytes) = joined.merged {
        let merged = Merged {
            digest: joined.digest.clone(),
            created: joined.created,
            bytes,
        };
        reply::reply(scope, &Frame::Merged(merged)).await;
    }
    let (digest, created) = (joined.digest, joined.created);
    // Rule 8.
    let ask = Publish {
        digest: digest.clone(),
        reference: frame.reference.clone(),
        tags: frame.tags.clone(),
        credential: frame.credential.map(|credential| Credential {
            username: credential.username,
            password: credential.password,
        }),
    };
    Ok(match sent(scope, &handle, &ask).await {
        Ok(digest) => Frame::Published(Published {
            provider,
            digest,
            created,
            reference: frame.reference,
            tags: frame.tags,
        }),
        Err(answer) => answer,
    })
}

/// Why a provider cannot do a publish to `registry`, or that it can.
enum Fit {
    /// It is connected, keeps a Store and uses the registry.
    Able,
    /// It is not connected now.
    Away,
    /// It keeps no Store.
    Storeless,
    /// It does not use the registry.
    Unlisted,
}

/// Whether `identity` can do a publish to `registry`, by what it said
/// of itself when it connected.
async fn fit(daemon: &Daemon, identity: &Identity, registry: &str) -> Fit {
    let Some(info) = daemon.live.capabilities(identity).await else {
        return Fit::Away;
    };
    if info.capabilities.images.store.is_none() {
        return Fit::Storeless;
    }
    if !info.capabilities.images.registries.iter().any(|listed| listed == registry) {
        return Fit::Unlisted;
    }
    Fit::Able
}

/// Rule 2 of [`serve`]: the providers a publish to `registry` may be
/// done on, in the daemon's record's order, or the answer that ends
/// the publish. The rules:
///
/// 1. **Named.** `NotFound` for a provider the daemon keeps no record
///    of; `Forbidden` for one no `publish` grant reaches, by its
///    identity or its tags; `Error` for one not connected now;
///    `Unsupported` for one with no Store; `RegistryUnlisted` for one
///    that does not use the registry. Else that provider alone.
/// 2. **Not named.** Every provider on record that a `publish` grant
///    reaches and that [`fit`]s. With none, `NoProvider`.
async fn candidates(daemon: &Daemon, standing: &Standing, named: Option<&Identity>, registry: &str) -> Result<Vec<Identity>, Frame> {
    let records = daemon.providers.all();
    // Rule 1.
    if let Some(identity) = named {
        let Some(record) = records.iter().find(|record| &record.identity == identity) else {
            return Err(Frame::NotFound);
        };
        if !judge::images::over(standing, Action::Publish, record) {
            return Err(Frame::Forbidden);
        }
        return match fit(daemon, identity, registry).await {
            Fit::Able => Ok(vec![identity.clone()]),
            Fit::Away => Err(Frame::Error(reply::failure(&"the provider is not connected now"))),
            Fit::Storeless => Err(Frame::Unsupported),
            Fit::Unlisted => Err(Frame::RegistryUnlisted),
        };
    }
    // Rule 2.
    let mut able = Vec::new();
    for record in records.iter().filter(|record| judge::images::over(standing, Action::Publish, record)) {
        if matches!(fit(daemon, &record.identity, registry).await, Fit::Able) {
            able.push(record.identity.clone());
        }
    }
    if able.is_empty() {
        return Err(Frame::NoProvider);
    }
    Ok(able)
}

/// Rule 5 of [`serve`]: of `able`, which is never empty, the provider
/// whose Store holds the most of the platforms `parts` take, the
/// first of `able` where several hold as many: the one the least has
/// to be brought onto.
fn most(able: &[Identity], held: &[(Identity, Vec<Listed>)], parts: &[Taken]) -> Identity {
    let holds = |identity: &Identity| -> usize {
        let Some((_, images)) = held.iter().find(|(holder, _)| holder == identity) else {
            return 0;
        };
        parts
            .iter()
            .map(|part| match images.iter().find(|image| image.digest == part.digest) {
                Some(image) => part.platforms.iter().filter(|platform| image.platforms.contains(platform)).count(),
                None => 0,
            })
            .sum()
    };
    let mut best = (&able[0], holds(&able[0]));
    for identity in &able[1..] {
        let count = holds(identity);
        if count > best.1 {
            best = (identity, count);
        }
    }
    best.0.clone()
}

/// Rule 8 of [`serve`]: the provider's publish, to its end. Each blob
/// the registry takes is sent as `Uploaded` as it comes; the digest
/// the registry holds the image by is the end, or the answer that
/// ends the publish: the provider's refusals of the same name, its
/// error in its words, and in the daemon's words an exchange that
/// broke, one that ended with no result, and the two refusals a
/// request that got this far cannot earn — an image the daemon does
/// not own there, and one held in part.
async fn sent(scope: &ScopeHandle, handle: &Handle, ask: &Publish) -> Result<String, Frame> {
    let failed = |error: &dyn std::fmt::Display| Frame::Error(reply::failure(&error.to_string()));
    let mut answers = publish::execute(handle, ask).await.map_err(|error| failed(&error))?;
    while let Some(answer) = answers.next().await {
        match answer {
            Ok(Sent::Blob(blob)) => {
                let uploaded = Uploaded {
                    digest: blob.digest,
                    bytes: blob.bytes,
                };
                reply::reply(scope, &Frame::Uploaded(uploaded)).await;
            }
            Ok(Sent::Published(published)) => return Ok(published.digest),
            Ok(_) => {}
            Err(publish::ExecuteStreamError::RegistryUnlisted) => return Err(Frame::RegistryUnlisted),
            Err(publish::ExecuteStreamError::Malformed) => return Err(Frame::Malformed),
            Err(publish::ExecuteStreamError::Unauthorized) => return Err(Frame::Unauthorized),
            Err(publish::ExecuteStreamError::Denied) => return Err(Frame::Denied),
            Err(publish::ExecuteStreamError::Unsupported) => return Err(Frame::Unsupported),
            Err(publish::ExecuteStreamError::Refused(error)) => return Err(Frame::Error(error)),
            Err(error) => return Err(failed(&error)),
        }
    }
    Err(failed(&"the provider ended the publish with no result"))
}

/// Every step told while the parts are brought in, sent on the
/// client's scope as the publish's own frame, in the order told, until
/// nobody is left to tell one.
async fn tell(scope: &ScopeHandle, mut said: mpsc::UnboundedReceiver<Step>) {
    while let Some(step) = said.recv().await {
        let frame = match step {
            Step::Blob(blob) => Frame::Blob(blob),
            Step::Added(added) => Frame::Added(added),
        };
        reply::reply(scope, &frame).await;
    }
}

/// A failure to bring a part in, as the answer of the same name.
fn refusal(failure: Failure) -> Frame {
    match failure {
        Failure::Unavailable => Frame::Unavailable,
        Failure::InsufficientCapacity => Frame::InsufficientCapacity,
        Failure::ProofFailed => Frame::ProofFailed,
        Failure::Malformed => Frame::Malformed,
        Failure::Unsupported => Frame::Unsupported,
        Failure::PlatformNotFound => Frame::PlatformNotFound,
        Failure::Error(error) => Frame::Error(error),
    }
}
