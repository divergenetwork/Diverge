//! Pushing an image into the daemon's Store on a provider.

use std::sync::Arc;

use diverge_sdk::daemon::endpoints::images::push::client::request::{self, Source};
use diverge_sdk::daemon::endpoints::images::push::server::response::{Added, Blob, Exists, Frame, Pushed};
use diverge_sdk::daemon::grant::images::Action;
use diverge_sdk::provider::client::{Manifest, OciStore};
use diverge_sdk::provider::endpoints::images::push::client::request::Frame as Ask;
use diverge_sdk::provider::endpoints::images::push::client::execute::serve as supply;
use diverge_sdk::provider::endpoints::images::push::server::response::Frame as Said;
use diverge_sdk::wire::client::handle::Handle;
use diverge_sdk::wire::credit::Receiver;
use diverge_sdk::wire::decode::Decode as _;
use diverge_sdk::wire::encode::{Encode as _, Writer};
use diverge_sdk::wire::frame::client::ClientFrame;
use diverge_sdk::wire::frame::server::ServerFrame;
use diverge_sdk::wire::server::channel::Channel;
use diverge_sdk::shared::containers::oci;
use diverge_sdk::shared::containers::request::Image;
use diverge_sdk::shared::containers::response::Source as Came;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;
use tokio::task::JoinSet;

use crate::containers::images;
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
///    finds: see [`from_daemon`].
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
/// 2. A push that ends with the image held sends one `Added`, of
///    every platform named, then `Pushed`. Every other end is the
///    provider's, as it said it.
async fn from_client(scope: &Arc<ScopeHandle>, handle: &Handle, frame: request::Frame) -> Frame {
    let platforms = frame.platforms.clone();
    let ask = Ask {
        image: frame.image,
        description: frame.description,
        platforms: frame.platforms,
    };
    match part(scope, handle, ask, None).await {
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
        Ended::Failed(answer) => answer,
    }
}

/// A push whose source is the daemon: the sources of the platforms
/// named found, and one push to the provider for each, all at once.
/// The rules:
///
/// 1. **Searched as a read is.** Every provider connected, the one
///    pushed to among them, and every remote daemon recorded is asked
///    at once which platforms of the image it can supply, as
///    [`images::search`] asks. The provider pushed to answers with
///    what the daemon owns there already and what its registries
///    serve, so a platform held there needs no other source.
/// 2. **Nothing moves until every platform is found.** The search is
///    waited for to its end. A platform named that no source supplies
///    is `Unavailable`: every source found is let go, and nothing is
///    asked of the provider.
/// 3. **One push for each source.** Each source's platforms are one
///    push to the provider, naming those and no other, answered
///    through that source alone: see [`part`]. They run at once, and
///    none waits for another.
/// 4. **Told as it happens.** Each blob a push pulls is sent as it
///    comes. Each push that ends with its platforms held is one
///    `Added`, sent as it ends: with the bytes and where the provider
///    took them from, or, where the provider held them all already,
///    with no bytes and `store`.
/// 5. **The end.** Every push is waited for, whatever another ended
///    as. All ended well, and at least one moved something: `Pushed`,
///    the bytes the largest any said — the image's size once the last
///    is counted. All held already: `Exists`. Otherwise the first
///    failure to end, as the provider said it; what the others added
///    stays, and was told.
async fn from_daemon(scope: &Arc<ScopeHandle>, handle: &Handle, frame: request::Frame, daemon: &Arc<Daemon>) -> Frame {
    // Rule 1.
    let reach = images::Reach::Full { except: None };
    let mut search = images::search(daemon, &frame.image, reach, &[], images::Wanted::Every(frame.platforms.clone()));
    let mut sources: Vec<(Arc<images::Source>, Vec<String>)> = Vec::new();
    while let Some(found) = search.next().await {
        match sources.iter_mut().find(|(source, _)| Arc::ptr_eq(source, &found.source)) {
            Some((_, platforms)) => platforms.extend(found.platforms),
            None => sources.push((found.source, found.platforms)),
        }
    }
    search.stop().await;
    // Rule 2.
    let found: usize = sources.iter().map(|(_, platforms)| platforms.len()).sum();
    if found != frame.platforms.len() {
        for (source, _) in sources {
            images::let_go(source);
        }
        return Frame::Unavailable;
    }
    // Rule 3.
    let mut pushes = JoinSet::new();
    for (source, platforms) in sources {
        let (scope, handle) = (Arc::clone(scope), handle.clone());
        let ask = Ask {
            image: frame.image.clone(),
            description: frame.description.clone(),
            platforms: platforms.clone(),
        };
        pushes.spawn(async move { (platforms, part(&scope, &handle, ask, Some(source)).await) });
    }
    // Rules 4 and 5.
    let mut pushed: Option<(u64, Came)> = None;
    let mut failed: Option<Frame> = None;
    while let Some(joined) = pushes.join_next().await {
        let (platforms, ended) = match joined {
            Ok(ended) => ended,
            Err(error) => {
                failed.get_or_insert(Frame::Error(reply::failure(&error)));
                continue;
            }
        };
        match ended {
            Ended::Pushed(said) => {
                let added = Added {
                    platforms,
                    bytes: Some(said.bytes),
                    source: said.source.clone(),
                };
                reply::reply(scope, &Frame::Added(added)).await;
                let bytes = pushed.as_ref().map_or(said.bytes, |(bytes, _)| said.bytes.max(*bytes));
                pushed = Some((bytes, said.source));
            }
            Ended::Exists(_) => {
                let added = Added {
                    platforms,
                    bytes: None,
                    source: Came::Store,
                };
                reply::reply(scope, &Frame::Added(added)).await;
            }
            Ended::Failed(answer) => {
                failed.get_or_insert(answer);
            }
        }
    }
    match (failed, pushed) {
        (Some(answer), _) => answer,
        (None, Some((bytes, source))) => Frame::Pushed(Pushed {
            digest: frame.image.digest,
            bytes,
            source,
            platforms: frame.platforms,
        }),
        (None, None) => Frame::Exists(Exists { digest: frame.image.digest }),
    }
}

/// How one push to the provider ended.
enum Ended {
    /// The platforms named are in the Store: the provider's word.
    Pushed(diverge_sdk::provider::endpoints::images::push::server::response::Pushed),
    /// The daemon held every platform named there already: the digest.
    Exists(String),
    /// Anything else, as the answer that says so.
    Failed(Frame),
}

/// One push to the provider, asked as the daemon itself, so that what
/// it brings in goes into the daemon's Store there. The rules:
///
/// 1. **The provider's asks are the source's.** With no `source`, the
///    client's: each is carried to the client and back by [`carry`].
///    With one, the provider's own push executor answers each from
///    that source's read scope, through [`Sourced`], and nothing is
///    opened on the client's scope.
/// 2. **The provider's answers, relayed.** Each blob pulled is sent to
///    the client as it comes. Its `Pushed` and its `Exists` end the
///    push well; its six refusals are relayed as they are; its error
///    is the `Error`, in its words; a push that would not open, an
///    exchange that broke, or one that ended with no result, is the
///    `Error`, in the daemon's.
/// 3. **The answering ends with the push**: whatever answer is still
///    on its way then is let go, and the source is released.
async fn part(scope: &Arc<ScopeHandle>, handle: &Handle, ask: Ask, source: Option<Arc<images::Source>>) -> Ended {
    let pulls = Arc::new(images::Pulls::new());
    if let Some(source) = &source {
        pulls.keep(Arc::clone(source)).await;
    }
    let ended = asked(scope, handle, ask, source.is_some(), &pulls).await;
    // Rule 3.
    pulls.release().await;
    ended
}

/// [`part`], between the source kept and the source released.
async fn asked(scope: &Arc<ScopeHandle>, handle: &Handle, ask: Ask, sourced: bool, pulls: &Arc<images::Pulls>) -> Ended {
    let failed = |error: &dyn std::fmt::Display| Ended::Failed(Frame::Error(reply::failure(&error.to_string())));
    let mut payload = Vec::new();
    if let Err(error) = ask.encode(&mut Writer::new(&mut payload)) {
        return failed(&error);
    }
    let mut remote = match handle.send_request(&payload).await {
        Ok(remote) => remote,
        Err(error) => return failed(&error),
    };
    // Rule 1.
    let answering = if sourced {
        let sourced = Arc::new(Sourced { pulls: Arc::clone(pulls) });
        tokio::spawn(supply(remote.request_receiver, handle.clone(), remote.scope, Arc::new(ask.image), sourced))
    } else {
        tokio::spawn(carry(Arc::clone(scope), handle.clone(), remote.scope, remote.request_receiver))
    };
    // Rule 2.
    let ended = loop {
        let Some(bytes) = remote.response_receiver.recv().await else {
            break failed(&"the connection to the provider ended with the push open");
        };
        let payload = match ServerFrame::decode(&bytes) {
            Ok(ServerFrame::Response { payload, .. }) => payload,
            Ok(ServerFrame::ResponseFinish { .. }) => break failed(&"the provider ended the push with no result"),
            Ok(_) => continue,
            Err(error) => break failed(&error),
        };
        match Said::decode(payload) {
            Ok(Said::Blob(blob)) => {
                reply::reply(
                    scope,
                    &Frame::Blob(Blob {
                        digest: blob.digest,
                        bytes: blob.bytes,
                    }),
                )
                .await;
            }
            Ok(Said::Pushed(pushed)) => break Ended::Pushed(pushed),
            Ok(Said::Exists(exists)) => break Ended::Exists(exists.digest),
            Ok(Said::InsufficientCapacity) => break Ended::Failed(Frame::InsufficientCapacity),
            Ok(Said::Unavailable) => break Ended::Failed(Frame::Unavailable),
            Ok(Said::ProofFailed) => break Ended::Failed(Frame::ProofFailed),
            Ok(Said::Malformed) => break Ended::Failed(Frame::Malformed),
            Ok(Said::Unsupported) => break Ended::Failed(Frame::Unsupported),
            Ok(Said::PlatformNotFound) => break Ended::Failed(Frame::PlatformNotFound),
            Ok(Said::Error(error)) => break Ended::Failed(Frame::Error(error)),
            Err(error) => break failed(&error),
        }
    };
    answering.abort();
    ended
}

/// A provider's asks on one push answered by the daemon, from the one
/// source the push is for, kept in `pulls`: read through it as a
/// run's asks are — see [`images`](crate::containers::images).
struct Sourced {
    /// The one scope held on the source.
    pulls: Arc<images::Pulls>,
}

impl OciStore for Sourced {
    async fn holds(&self, image: &Image) -> Option<String> {
        self.pulls.held().await.map(|_| images::name(&image.digest))
    }

    type Blob = images::Blob;

    async fn manifest(&self, _image: &Image, digest: &str) -> Option<Manifest> {
        images::manifest(&self.pulls, digest).await
    }

    async fn blob(&self, _image: &Image, digest: &str) -> Option<Self::Blob> {
        images::blob(&self.pulls, digest).await
    }

    async fn prove(&self, _image: &Image, request: &oci::prove::request::Request) -> Option<[u8; 32]> {
        images::prove(&self.pulls, request).await
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
