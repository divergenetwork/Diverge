//! Pushing an image into the daemon's Store on a provider.

use std::sync::Arc;

use diverge_sdk::daemon::endpoints::daemons::DaemonId;
use diverge_sdk::daemon::endpoints::images::push::client::request::{self, Source};
use diverge_sdk::daemon::endpoints::images::push::server::response::{Blob, Exists, Frame, Pushed};
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
use diverge_sdk::shared::Identity;
use diverge_sdk::shared::containers::oci;
use diverge_sdk::shared::containers::request::Image;
use diverge_sdk::shared::permission;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;
use tokio::sync::Mutex;
use tokio::task::JoinSet;

use crate::containers::images;
use crate::daemon::Daemon;
use crate::judge::{self, Standing, Who};
use crate::serve::reply;
use crate::store;
use crate::volumes;

/// Answer the push — each blob pulled as it comes, then how it ended —
/// and finish the scope.
pub async fn handle(scope: ScopeHandle, frame: request::Frame, who: Who, daemon: &Arc<Daemon>) {
    let scope = Arc::new(scope);
    let answer = match serve(&scope, frame, who, daemon).await {
        Ok(answer) => answer,
        Err(error) => Frame::Error(reply::failure(&error)),
    };
    reply::reply(&scope, &answer).await;
    scope.send_response_finish().await;
}

/// The answer that ends the push, every blob pulled sent on the scope
/// before it. The rules:
///
/// 1. `Forbidden` with no `push` grant over images at all.
/// 2. `NotFound` for a provider the daemon keeps no record of;
///    `Forbidden` for one no `push` grant reaches, by its identity or
///    its tags.
/// 3. **The source is judged as what it is**, by [`judged`], before
///    anything is asked of anyone: a provider by `read` over it, a
///    remote daemon by `relay` over the route's first record, the
///    client by nothing.
/// 4. Then the provider is asked, as the daemon itself, so the image
///    goes into the daemon's Store there. `Error` for a provider not
///    connected now, or a push that would not open.
/// 5. **The provider's asks are the source's.** Where the source is
///    the client, every channel the provider opens on its push is
///    carried to the client and back by [`carry`], and the daemon
///    answers none of them itself. Where it is a provider or a remote
///    daemon, the provider's own push executor answers each, from
///    the one read scope [`Sourced`] opens there at the first ask, and nothing is opened
///    on the client's scope.
/// 6. **The provider's answers, relayed.** Each blob pulled is relayed
///    as it comes. The provider's `Pushed`, `Exists` and its five
///    refusals are relayed as they are; its error is the `Error`, in
///    its words; an exchange that broke, or ended with no result, is
///    the `Error`, in the daemon's.
/// 7. **The answering ends with the push**: whatever answer is still
///    on its way then is let go, and a source that was opened is
///    released.
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
    // Rule 3.
    let from = match judged(&mut conn, &standing, daemon, &records, &frame.source).await? {
        Ok(from) => from,
        Err(refusal) => return Ok(refusal),
    };
    drop(conn);
    scope.allowed().await;
    // Rule 4.
    let handle = match volumes::handle(daemon, &frame.provider).await {
        Ok(handle) => handle,
        Err(fail) => return Ok(Frame::Error(fail.error())),
    };
    let image = frame.image.clone();
    let ask = Ask {
        image: frame.image,
        description: frame.description,
    };
    let mut payload = Vec::new();
    if let Err(error) = ask.encode(&mut Writer::new(&mut payload)) {
        return Ok(Frame::Error(reply::failure(&error)));
    }
    let mut remote = match handle.send_request(&payload).await {
        Ok(remote) => remote,
        Err(error) => return Ok(Frame::Error(reply::failure(&error))),
    };
    // Rule 5.
    let pulls = Arc::new(images::Pulls::new());
    let answering = match from {
        From::Client => tokio::spawn(carry(Arc::clone(scope), handle.clone(), remote.scope, remote.request_receiver)),
        from => {
            let sourced = Arc::new(Sourced {
                daemon: Arc::clone(daemon),
                pulls: Arc::clone(&pulls),
                from,
                tried: Mutex::new(false),
            });
            tokio::spawn(supply(remote.request_receiver, handle.clone(), remote.scope, Arc::new(image), sourced))
        }
    };
    // Rule 6.
    let answer = loop {
        let Some(bytes) = remote.response_receiver.recv().await else {
            break Frame::Error(reply::failure(&"the connection to the provider ended with the push open"));
        };
        let payload = match ServerFrame::decode(&bytes) {
            Ok(ServerFrame::Response { payload, .. }) => payload,
            Ok(ServerFrame::ResponseFinish { .. }) => break Frame::Error(reply::failure(&"the provider ended the push with no result")),
            Ok(_) => continue,
            Err(error) => break Frame::Error(reply::failure(&error)),
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
            Ok(Said::Pushed(pushed)) => {
                break Frame::Pushed(Pushed {
                    digest: pushed.digest,
                    bytes: pushed.bytes,
                    source: pushed.source,
                });
            }
            Ok(Said::Exists(exists)) => break Frame::Exists(Exists { digest: exists.digest }),
            Ok(Said::InsufficientCapacity) => break Frame::InsufficientCapacity,
            Ok(Said::Unavailable) => break Frame::Unavailable,
            Ok(Said::ProofFailed) => break Frame::ProofFailed,
            Ok(Said::Malformed) => break Frame::Malformed,
            Ok(Said::Unsupported) => break Frame::Unsupported,
            Ok(Said::Error(error)) => break Frame::Error(error),
            Err(error) => break Frame::Error(reply::failure(&error)),
        }
    };
    // Rule 7.
    answering.abort();
    pulls.release().await;
    Ok(answer)
}

/// Where the image comes from, once the caller has been judged to
/// reach it.
enum From {
    /// The client.
    Client,
    /// A provider on record, by its identity.
    Provider(Identity),
    /// A remote daemon's record, and the route beyond it.
    Daemon(store::daemons::Record, Vec<DaemonId>),
}

/// The request's source, judged: rule 3 of [`serve`]. The outer error
/// is the store's; the inner is the answer that ends the push. The
/// rules:
///
/// 1. The client is its own source, and needs no grant.
/// 2. A provider named is one of `records`, else `NotFound`; and one a
///    `providers` grant reaches with `read`, else `Forbidden`.
/// 3. A route names at least one daemon, else `Error`. Its first ID is
///    a record of this daemon's, else `Error`, as a relay refuses one;
///    and one a `daemons` grant reaches with `relay`, else
///    `Forbidden`. The IDs after it are not judged here: each is the
///    next daemon's to judge.
async fn judged(conn: &mut sqlx::PgConnection, standing: &Standing, daemon: &Arc<Daemon>, records: &[store::providers::Record], source: &Source) -> Result<Result<From, Frame>, store::Error> {
    Ok(match source {
        // Rule 1.
        Source::Client => Ok(From::Client),
        // Rule 2.
        Source::Provider(identity) => match records.iter().find(|record| &record.identity == identity) {
            None => Err(Frame::NotFound),
            Some(record) if !judge::providers::over(standing, permission::providers::Over::Read, record) => Err(Frame::Forbidden),
            Some(record) => Ok(From::Provider(record.identity.clone())),
        },
        // Rule 3.
        Source::Daemon(route) => {
            let Some((first, rest)) = route.0.split_first() else {
                return Ok(Err(Frame::Error(reply::failure(&"the source names no daemon"))));
            };
            let Some(record) = store::daemons::by_id(conn, *first, false).await? else {
                return Ok(Err(Frame::Error(reply::failure(&"the source's first daemon is none the daemon keeps a record of"))));
            };
            let connected = daemon.live.connected_daemons().await.contains(&record.id);
            if !judge::daemons::over(standing, permission::daemons::Over::Relay, &record.report(connected)) {
                return Ok(Err(Frame::Forbidden));
            }
            Ok(From::Daemon(record, rest.to_vec()))
        }
    })
}

/// A provider's asks answered from one source: a provider, or a remote
/// daemon, and no other. The rules:
///
/// 1. **Opened at the first ask, once.** The read scope on the source
///    is opened when the provider first asks for anything, and kept in
///    `pulls`. A source that does not answer held is not asked again:
///    every ask is then answered not held.
/// 2. **Read through it**, as a run's asks are: see
///    [`images`](crate::containers::images).
struct Sourced {
    /// The daemon, for its connections.
    daemon: Arc<Daemon>,
    /// The one scope held on the source.
    pulls: Arc<images::Pulls>,
    /// The source.
    from: From,
    /// Whether the source has been asked whether it holds the image.
    tried: Mutex<bool>,
}

impl Sourced {
    /// Rule 1: the source asked, where it has not been.
    async fn opened(&self, image: &Image) {
        let mut tried = self.tried.lock().await;
        if *tried {
            return;
        }
        *tried = true;
        let source = match &self.from {
            From::Client => None,
            From::Provider(identity) => images::on_provider(&self.daemon, image, identity).await,
            From::Daemon(record, rest) => images::on_daemon(&self.daemon, image, record, rest.clone()).await,
        };
        if let Some(source) = source {
            self.pulls.keep(source).await;
        }
    }
}

impl OciStore for Sourced {
    async fn holds(&self, image: &Image) -> Option<String> {
        self.opened(image).await;
        self.pulls.held().await.map(|_| images::name(&image.digest))
    }

    type Blob = images::Blob;

    async fn manifest(&self, image: &Image, digest: &str) -> Option<Manifest> {
        self.opened(image).await;
        images::manifest(&self.pulls, digest).await
    }

    async fn blob(&self, image: &Image, digest: &str) -> Option<Self::Blob> {
        self.opened(image).await;
        images::blob(&self.pulls, digest).await
    }

    async fn prove(&self, image: &Image, request: &oci::prove::request::Request) -> Option<[u8; 32]> {
        self.opened(image).await;
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
