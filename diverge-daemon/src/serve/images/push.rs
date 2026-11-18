//! Pushing an image into the daemon's Store on a provider.

use std::sync::Arc;

use diverge_sdk::daemon::endpoints::images::push::client::request::{self, Source};
use diverge_sdk::daemon::endpoints::images::push::server::response::{Added, Exists, Frame, Pushed};
use diverge_sdk::daemon::grant::images::Action;
use diverge_sdk::provider::endpoints::images::push::client::request::Frame as Ask;
use diverge_sdk::wire::client::handle::Handle;
use diverge_sdk::wire::credit::Receiver;
use diverge_sdk::wire::frame::client::ClientFrame;
use diverge_sdk::wire::frame::server::ServerFrame;
use diverge_sdk::wire::server::channel::Channel;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use super::bring::{self, Brought, Ended, Failure, Step};
use crate::daemon::Daemon;
use crate::judge::{self, Standing, Who};
use crate::serve::reply;
use crate::store;
use crate::volumes;

/// Answer the push — each blob pulled and each set of platforms added
/// as it comes, then how it ended — and finish the scope.
pub async fn handle(scope: ScopeHandle, frame: request::Frame, who: Who, daemon: &Arc<Daemon>) {
    let scope = Arc::new(scope);
    let answer = match serve(&scope, frame, who, daemon).await {
        Ok(answer) => answer,
        Err(error) => Frame::Error(reply::failure(&error)),
    };
    reply::reply(&scope, &answer).await;
    scope.send_response_finish().await;
}

/// The answer that ends the push, every blob pulled and every set of
/// platforms added sent on the scope before it. The rules:
///
/// 1. `Forbidden` with no `push` grant over images at all.
/// 2. `NotFound` for a provider the daemon keeps no record of;
///    `Forbidden` for one no `push` grant reaches, by its identity or
///    its tags.
/// 3. **The source is judged by nothing.** The client is the source
///    of its own image; the daemon asks as itself, as it does for a
///    run.
/// 4. `Error` for a provider not connected now.
/// 5. **From the client**, one push to the provider: see
///    [`from_client`]. **From the daemon**, one for each source it
///    finds: see [`from_daemon`] and [`bring`].
async fn serve(scope: &Arc<ScopeHandle>, frame: request::Frame, who: Who, daemon: &Arc<Daemon>) -> Result<Frame, store::Error> {
    let mut conn = daemon.store.acquire().await?;
    let Some(standing) = Standing::of(&mut conn, who).await? else {
        return Ok(Frame::Forbidden);
    };
    // Rule 1.
    if !judge::images::holds(&standing, Action::Push) {
        return Ok(Frame::Forbidden);
    }
    // Rule 2.
    let records = daemon.providers.all();
    let Some(record) = records.iter().find(|record| record.identity == frame.provider) else {
        return Ok(Frame::NotFound);
    };
    if !judge::images::over(&standing, Action::Push, record) {
        return Ok(Frame::Forbidden);
    }
    drop(conn);
    scope.allowed().await;
    // Rule 4.
    let handle = match volumes::handle(daemon, &frame.provider).await {
        Ok(handle) => handle,
        Err(fail) => return Ok(Frame::Error(fail.error())),
    };
    // Rule 5.
    Ok(match frame.source {
        Source::Client => from_client(scope, &handle, frame).await,
        Source::Daemon => from_daemon(scope, &handle, frame, daemon).await,
    })
}

/// A push whose source is the client: one push to the provider, every
/// ask of which is the client's. The rules:
///
/// 1. Every channel the provider opens on its push is carried to the
///    client and back by [`carry`], and the daemon answers none of
///    them itself.
/// 2. Each blob pulled is sent as it comes. A push that ends with the
///    image held sends one `Added`, of every platform named, then
///    `Pushed`. Every other end is the provider's, as it said it.
async fn from_client(scope: &Arc<ScopeHandle>, handle: &Handle, frame: request::Frame) -> Frame {
    let platforms = frame.platforms.clone();
    let ask = Ask {
        image: frame.image,
        description: frame.description,
        platforms: frame.platforms,
    };
    let (steps, said) = mpsc::unbounded_channel();
    let pushing = async move {
        bring::put(handle, ask, &steps, |asks, remote, _| tokio::spawn(carry(Arc::clone(scope), handle.clone(), remote, asks))).await
    };
    let (ended, ()) = tokio::join!(pushing, tell(scope, said));
    match ended {
        Ended::Pushed(pushed) => {
            let added = Added {
                platforms: platforms.clone(),
                bytes: Some(pushed.bytes),
                source: pushed.source.clone(),
            };
            reply::reply(scope, &Frame::Added(added)).await;
            Frame::Pushed(Pushed {
                digest: pushed.digest,
                bytes: pushed.bytes,
                source: pushed.source,
                platforms,
            })
        }
        Ended::Exists(digest) => Frame::Exists(Exists { digest }),
        Ended::Failed(failure) => refusal(failure),
    }
}

/// A push whose source is the daemon: the platforms named brought in
/// from the sources the daemon finds, as [`bring::bring`] states —
/// one push to the provider for each source, all at once, nothing
/// moved until every platform has one. What it tells is sent as it
/// comes. The end: `Pushed` where every push ended well and at least
/// one moved something; `Exists` where the provider held every
/// platform already; otherwise the first failure to end, as the
/// provider said it, what the others added staying added.
async fn from_daemon(scope: &Arc<ScopeHandle>, handle: &Handle, frame: request::Frame, daemon: &Arc<Daemon>) -> Frame {
    let (steps, said) = mpsc::unbounded_channel();
    let bringing = async {
        let brought = bring::bring(daemon, handle, &frame.image, frame.description.as_ref(), &frame.platforms, &steps).await;
        drop(steps);
        brought
    };
    let (brought, ()) = tokio::join!(bringing, tell(scope, said));
    match brought {
        Brought::Pushed { bytes, source } => Frame::Pushed(Pushed {
            digest: frame.image.digest,
            bytes,
            source,
            platforms: frame.platforms,
        }),
        Brought::Exists => Frame::Exists(Exists { digest: frame.image.digest }),
        Brought::Failed(failure) => refusal(failure),
    }
}

/// Every step told, sent on the client's scope as the push's own
/// frame, in the order told, until nobody is left to tell one.
async fn tell(scope: &ScopeHandle, mut said: mpsc::UnboundedReceiver<Step>) {
    while let Some(step) = said.recv().await {
        let frame = match step {
            Step::Blob(blob) => Frame::Blob(blob),
            Step::Added(added) => Frame::Added(added),
        };
        reply::reply(scope, &frame).await;
    }
}

/// A push's failure, as the answer of the same name.
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

/// Every ask the provider opens on its push, carried to the client and
/// its answer carried back, until the provider's scope stops yielding
/// asks. The rules:
///
/// 1. **As it came.** Each ask is opened on the client's scope with
///    the provider's own payload, unread, and each answer is one task
///    of its own: several are carried at once.
/// 2. **Under the provider's number.** Each side minted its own channel
///    numbers; the pair lives in the task that carries that answer.
/// 3. **Ends with its owner.** Let go, this lets go of every answer it
///    is still carrying.
async fn carry(scope: Arc<ScopeHandle>, handle: Handle, remote: u32, mut asks: Receiver) {
    let mut answers = JoinSet::new();
    loop {
        let bytes = tokio::select! {
            received = asks.recv() => match received {
                Some(bytes) => bytes,
                None => break,
            },
            Some(_) = answers.join_next(), if !answers.is_empty() => continue,
        };
        let Ok(ServerFrame::ChannelRequest { channel, payload, .. }) = ServerFrame::decode(&bytes) else {
            continue;
        };
        let opened = scope.send_channel_request(payload).await;
        answers.spawn(answer(handle.clone(), remote, channel, opened));
    }
    while answers.join_next().await.is_some() {}
}

/// The client's answer on one channel, sent to the provider under the
/// provider's number, to its end. The rules:
///
/// 1. **As it came.** Every frame of the answer is sent on as the
///    client wrote it.
/// 2. **Always finished.** The client's finish is the provider's
///    finish. So is the answer ending without one — a client that
///    reads no asks, that went away, or whose scope ended: the
///    provider is sent the finish all the same, which after nothing is
///    the wire's "not held", and after part of a blob is a blob whose
///    bytes do not hash to its digest. A provider is never left
///    waiting on an ask nobody will answer.
/// 3. The channel is held here for as long as it is carried, since its
///    dropping is its closing on the client's side.
async fn answer(handle: Handle, remote: u32, channel: u32, mut opened: Channel) {
    while let Some(bytes) = opened.response_receiver.recv().await {
        match ClientFrame::decode(&bytes) {
            Ok(ClientFrame::ChannelResponse { payload, .. }) => {
                if handle.send_channel_response(remote, channel, payload).await.is_err() {
                    return;
                }
            }
            Ok(ClientFrame::ChannelResponseFinish { .. }) => break,
            _ => {}
        }
    }
    let _ = handle.send_channel_response_finish(remote, channel).await;
}
