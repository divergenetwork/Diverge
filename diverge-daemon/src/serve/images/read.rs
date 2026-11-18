//! Whether the daemon can supply an image, and everything read of it:
//! one scope.

use std::sync::Arc;

use diverge_sdk::CHUNK_SIZE;
use diverge_sdk::daemon::endpoints::images::read::client::{channel_request, request};
use diverge_sdk::daemon::endpoints::images::read::server::response::Frame;
use diverge_sdk::shared::containers::oci;
use diverge_sdk::shared::permission;
use diverge_sdk::wire::decode::Decode as _;
use diverge_sdk::wire::encode::{Encode, Writer};
use diverge_sdk::wire::frame::client::ClientFrame;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;
use futures_util::StreamExt as _;
use tokio::task::JoinSet;

use crate::containers::images;
use crate::daemon::Daemon;
use crate::judge::{self, Standing, Who};
use crate::serve::reply;
use crate::store;
use crate::store::image_icons::Which;

/// Say which platforms of the image are held, as each is found, and
/// serve it, every ask the caller opens on it, until the caller
/// cancels.
///
/// The rules:
///
/// 1. **Forbidden.** An account that holds neither `read` over a
///    provider nor `relay` over a remote daemon is answered `Forbidden`,
///    and nothing is asked.
/// 2. **The reach.** The providers asked are those a `providers` grant
///    reaches with `read`; the remote daemons, the records a `daemons`
///    grant reaches with `relay`, as a list reports them.
/// 3. **Held, as each is found.** The sources are asked as
///    [`images::search`] asks them, for the platforms the request
///    names, `ignore` passed on. Each source that adds platforms is
///    kept and is one `Held`, carrying what it added; a search that
///    finds none is `NotFound`. A store that cannot be read is
///    `Error`.
/// 3a. **Settled, once.** When the search ends — every platform found,
///    every source answered, or the caller's `Enough` — `Settled` is
///    sent, and no `Held` after it. The search runs beside the serving:
///    a channel opened after the first `Held` is answered while later
///    sources are still being asked.
/// 4. **Served through them.** Each channel the caller opens is answered
///    on its own task, through the sources kept as [`images::manifest`],
///    [`images::blob`] and [`images::prove`] pull through them: a
///    manifest with at most one frame then the finish; a blob with its
///    pieces, each split at [`CHUNK_SIZE`], then the finish; a proof
///    with one frame of its 32 bytes then the finish — the empty finish
///    being a part no source answered. The image's document is
///    answered with exactly one frame then the finish, as
///    [`images::through`] reads it: from the daemon's record of the
///    digest, else through a source. Each of the image's three
///    icons is answered the same way, as [`images::icon`] reads it:
///    from the daemon's record of the digest and the icon, else
///    through a source, whose answer is carried as it came. A
///    channel request that will not decode is finished with nothing.
/// 5. **The end.** The caller's `Cancel`, the caller gone, or the
///    caller reading no more of the scope ends the loop; every answer
///    still in flight is finished, save that a caller reading no more
///    is not waited on, and then the scope. The search is stopped and
///    every source cancelled after the finish.
pub async fn handle(scope: ScopeHandle, frame: request::Frame, who: Who, daemon: &Arc<Daemon>) {
    let digest = frame.image.digest.clone();
    let mut search = match open(&scope, frame, who, daemon).await {
        Ok(Ok(search)) => search,
        Ok(Err(answer)) => {
            reply::reply(&scope, &answer).await;
            scope.send_response_finish().await;
            return;
        }
        Err(error) => {
            reply::reply(&scope, &Frame::Error(reply::failure(&error))).await;
            scope.send_response_finish().await;
            return;
        }
    };
    // Rule 3.
    let Some(first) = search.next().await else {
        search.stop().await;
        reply::reply(&scope, &Frame::NotFound).await;
        scope.send_response_finish().await;
        return;
    };
    let pulls = Arc::new(images::Pulls::new());
    pulls.keep(first.source).await;
    reply::reply(&scope, &Frame::Held { platforms: first.platforms }).await;
    let mut settled = false;
    // Rule 4.
    let scope = Arc::new(scope);
    let mut tasks = JoinSet::new();
    loop {
        let bytes = tokio::select! {
            // Rules 3 and 3a.
            found = search.next(), if !settled => {
                match found {
                    Some(found) => {
                        pulls.keep(found.source).await;
                        reply::reply(&scope, &Frame::Held { platforms: found.platforms }).await;
                    }
                    None => {
                        settled = true;
                        reply::reply(&scope, &Frame::Settled).await;
                    }
                }
                continue;
            }
            received = scope.recv_channel_request() => match received {
                Some(bytes) => bytes,
                None => break,
            },
            () = scope.responses_gone() => break,
            Some(_) = tasks.join_next(), if !tasks.is_empty() => continue,
        };
        let Ok(ClientFrame::ChannelRequest { channel, payload, .. }) = ClientFrame::decode(&bytes) else {
            continue;
        };
        match channel_request::Frame::decode(payload) {
            Ok(channel_request::Frame::OciManifest(request)) => {
                tasks.spawn(manifest(Arc::clone(&scope), Arc::clone(&pulls), channel, request.digest));
            }
            Ok(channel_request::Frame::OciBlob(request)) => {
                tasks.spawn(blob(Arc::clone(&scope), Arc::clone(&pulls), channel, request.digest));
            }
            Ok(channel_request::Frame::OciProve(request)) => {
                tasks.spawn(prove(Arc::clone(&scope), Arc::clone(&pulls), channel, request));
            }
            Ok(channel_request::Frame::Info) => {
                tasks.spawn(info(Arc::clone(&scope), Arc::clone(daemon), Arc::clone(&pulls), channel, digest.clone()));
            }
            Ok(channel_request::Frame::Cancel) => break,
            Ok(channel_request::Frame::Enough) => {
                search.enough();
                scope.send_channel_response_finish(channel).await;
            }
            Ok(channel_request::Frame::IconSvg) => {
                tasks.spawn(icon(Arc::clone(&scope), Arc::clone(daemon), Arc::clone(&pulls), channel, digest.clone(), Which::Svg));
            }
            Ok(channel_request::Frame::Icon512) => {
                tasks.spawn(icon(Arc::clone(&scope), Arc::clone(daemon), Arc::clone(&pulls), channel, digest.clone(), Which::Png512));
            }
            Ok(channel_request::Frame::Icon64) => {
                tasks.spawn(icon(Arc::clone(&scope), Arc::clone(daemon), Arc::clone(&pulls), channel, digest.clone(), Which::Png64));
            }
            Err(_) => scope.send_channel_response_finish(channel).await,
        }
    }
    // Rule 5.
    let gone = tokio::select! {
        () = drain(&mut tasks) => false,
        () = scope.responses_gone() => true,
    };
    if gone {
        tasks.abort_all();
        drain(&mut tasks).await;
    }
    scope.send_response_finish().await;
    search.stop().await;
    pulls.release().await;
}

/// Rules 1 to 3: the search begun, or the answer that ends the scope.
async fn open(scope: &ScopeHandle, frame: request::Frame, who: Who, daemon: &Arc<Daemon>) -> Result<Result<images::Search, Frame>, store::Error> {
    let mut conn = daemon.store.acquire().await?;
    // Rule 1.
    let Some(standing) = Standing::of(&mut conn, who).await? else {
        return Ok(Err(Frame::Forbidden));
    };
    let over_providers = judge::providers::holds(&standing, permission::providers::Over::Read);
    let over_daemons = judge::daemons::holds(&standing, permission::daemons::Over::Relay);
    if !over_providers && !over_daemons {
        return Ok(Err(Frame::Forbidden));
    }
    scope.allowed().await;
    // Rule 2.
    let providers = judge::providers::reached(&standing, permission::providers::Over::Read, &daemon.providers.all());
    let daemons = if over_daemons {
        let records = store::daemons::all(&mut conn).await?;
        judge::daemons::reached(&standing, permission::daemons::Over::Relay, records, &daemon.live.connected_daemons().await)
    } else {
        Vec::new()
    };
    drop(conn);
    // Rule 3.
    let reach = images::Reach::Granted { providers, daemons };
    let wanted = frame.platforms.map_or(images::Wanted::Any, images::Wanted::Every);
    Ok(Ok(images::search(daemon, &frame.image, reach, &frame.ignore, wanted)))
}

/// Every task of `tasks` waited for, to the last.
async fn drain(tasks: &mut JoinSet<()>) {
    while tasks.join_next().await.is_some() {}
}

/// At most one frame — the manifest's media type and bytes — then the
/// finish.
async fn manifest(scope: Arc<ScopeHandle>, pulls: Arc<images::Pulls>, channel: u32, digest: String) {
    if let Some(manifest) = images::manifest(&pulls, &digest).await {
        let frame = oci::manifest::response::Frame {
            media_type: &manifest.media_type,
            body: &manifest.body,
        };
        respond(&scope, channel, &frame).await;
    }
    scope.send_channel_response_finish(channel).await;
}

/// The blob's pieces, each split at the chunk size, then the finish.
async fn blob(scope: Arc<ScopeHandle>, pulls: Arc<images::Pulls>, channel: u32, digest: String) {
    if let Some(mut pieces) = images::blob(&pulls, &digest).await {
        while let Some(piece) = pieces.next().await {
            for body in piece.chunks(CHUNK_SIZE) {
                respond(&scope, channel, &oci::blob::response::Frame { body }).await;
            }
        }
    }
    scope.send_channel_response_finish(channel).await;
}

/// At most one frame — the proof — then the finish.
async fn prove(scope: Arc<ScopeHandle>, pulls: Arc<images::Pulls>, channel: u32, request: oci::prove::request::Request) {
    if let Some(proof) = images::prove(&pulls, &request).await {
        respond(&scope, channel, &oci::prove::response::Frame { proof }).await;
    }
    scope.send_channel_response_finish(channel).await;
}

/// Exactly one frame — the document, that the image ships none, an
/// error, or that the file is too large — then the finish.
async fn info(scope: Arc<ScopeHandle>, daemon: Arc<Daemon>, pulls: Arc<images::Pulls>, channel: u32, digest: String) {
    respond(&scope, channel, &images::through(&daemon, &pulls, &digest).await).await;
    scope.send_channel_response_finish(channel).await;
}

/// Exactly one frame — the icon's bytes, that the image ships none, an
/// error, or that the file is too large — then the finish.
async fn icon(scope: Arc<ScopeHandle>, daemon: Arc<Daemon>, pulls: Arc<images::Pulls>, channel: u32, digest: String, which: Which) {
    respond(&scope, channel, &images::icon(&daemon, &pulls, &digest, which).await).await;
    scope.send_channel_response_finish(channel).await;
}

/// One channel response, when it encoded; one that will not is not
/// sent, and the finish that follows says so.
async fn respond<T: Encode>(scope: &ScopeHandle, channel: u32, frame: &T) {
    let mut buffer = Vec::new();
    if frame.encode(&mut Writer::new(&mut buffer)).is_ok() {
        scope.send_channel_response(channel, &buffer).await;
    }
}
