//! Merging images into one, in the daemon's Store on a provider.

use std::sync::Arc;

use diverge_sdk::daemon::endpoints::images::merge::client::request;
use diverge_sdk::daemon::endpoints::images::merge::server::response::{Frame, Merged};
use diverge_sdk::daemon::grant::images::Action;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;
use futures_util::future::join_all;
use tokio::sync::mpsc;

use super::bring::{Failure, Step};
use super::join::{self, Stopped, Taken};
use crate::daemon::Daemon;
use crate::judge::{self, Standing, Who};
use crate::serve::reply;
use crate::store;
use crate::volumes;

/// Answer the merge — what happened as it happened, then how it
/// ended — and finish the scope.
pub async fn handle(scope: ScopeHandle, frame: request::Frame, who: Who, daemon: &Arc<Daemon>) {
    let answer = match serve(&scope, frame, who, daemon).await {
        Ok(answer) => answer,
        Err(error) => Frame::Error(reply::failure(&error)),
    };
    reply::reply(&scope, &answer).await;
    scope.send_response_finish().await;
}

/// The answer that ends the merge, everything that happened before it
/// sent on the scope as it happened. The rules:
///
/// 1. `Forbidden` with no `merge` grant over images at all.
/// 2. **The provider named**: `NotFound` for one the daemon keeps no
///    record of; `Forbidden` for one no `merge` grant reaches, by its
///    identity or its tags; `Error` for one not connected now;
///    `Unsupported` for one with no Store.
/// 3. **The platforms of each image**, by [`join::taken`], every
///    image at once: those it names, or every platform its manifest
///    names. `Unavailable` for a digest whose manifest no source
///    supplies.
/// 4. `PlatformRepeated` for a platform two images take.
/// 5. **Brought in and merged**, by [`join::join`]: each blob pulled
///    told as `Blob`, each set of platforms that landed as `Added`,
///    and its end the answer — `Merged`, or why not.
async fn serve(scope: &ScopeHandle, frame: request::Frame, who: Who, daemon: &Arc<Daemon>) -> Result<Frame, store::Error> {
    let standing = {
        let mut conn = daemon.store.acquire().await?;
        Standing::of(&mut conn, who).await?
    };
    // Rule 1.
    let Some(standing) = standing else {
        return Ok(Frame::Forbidden);
    };
    if !judge::images::holds(&standing, Action::Merge) {
        return Ok(Frame::Forbidden);
    }
    // Rule 2.
    let records = daemon.providers.all();
    let Some(record) = records.iter().find(|record| record.identity == frame.provider) else {
        return Ok(Frame::NotFound);
    };
    if !judge::images::over(&standing, Action::Merge, record) {
        return Ok(Frame::Forbidden);
    }
    let Some(info) = daemon.live.capabilities(&frame.provider).await else {
        return Ok(Frame::Error(reply::failure(&"the provider is not connected now")));
    };
    if info.capabilities.images.store.is_none() {
        return Ok(Frame::Unsupported);
    }
    scope.allowed().await;
    // Rule 3.
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
    let handle = match volumes::handle(daemon, &frame.provider).await {
        Ok(handle) => handle,
        Err(fail) => return Ok(Frame::Error(fail.error())),
    };
    // Rule 5.
    let (steps, said) = mpsc::unbounded_channel();
    let (joined, ()) = tokio::join!(join::join(daemon, &frame.provider, &handle, &held, &parts, Some(&frame.description), steps), tell(scope, said));
    Ok(match joined {
        Ok(joined) => Frame::Merged(Merged {
            digest: joined.digest,
            created: joined.created,
            bytes: joined.merged.flatten(),
        }),
        Err(Stopped::Brought(failure)) => refusal(failure),
        Err(Stopped::PlatformRepeated) => Frame::PlatformRepeated,
        Err(Stopped::InsufficientCapacity) => Frame::InsufficientCapacity,
        Err(Stopped::Unsupported) => Frame::Unsupported,
        Err(Stopped::Error(error)) => Frame::Error(error),
    })
}

/// Every step told while the images are brought in, sent on the
/// client's scope as the merge's own frame, in the order told, until
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

/// A failure to bring an image in, as the answer of the same name.
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
