//! A request through one of a tool program's two doors — its frontend
//! or its API — served for as long as the scope is open. The two are
//! one exchange beneath two prefixes, each under a grant action of its
//! own and each declared by its own member of the image's document, so
//! everything here is told which [`Door`] it is.

use std::convert::Infallible;
use std::sync::Arc;

use bytes::Bytes;
use diverge_sdk::daemon::endpoints::standalone_tools::frontend::client::channel_request;
use diverge_sdk::shared::containers::frontend;
use diverge_sdk::shared::containers::request::Image;
use diverge_sdk::shared::error::Error;
use diverge_sdk::shared::mcp::attestation::Caller;
use diverge_sdk::shared::mcp;
use diverge_sdk::wire::decode::Decode as _;
use diverge_sdk::wire::encode::Encode;
use diverge_sdk::wire::frame::client::ClientFrame;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;
use futures_util::StreamExt as _;
use futures_util::stream::FuturesUnordered;
use tokio::sync::mpsc;

use crate::containers::{Door, Opened, images};
use crate::daemon::Daemon;
use crate::judge::{Container, Standing, Who};
use crate::serve::reply;
use crate::store;

/// What a request through a door is answered with, whichever family's
/// and whichever door's: the four frames every one of them has beside
/// the program's own answer, so that one handler serves a family's
/// frontend and its API alike.
pub trait Answer: From<frontend::output::Owned> + Encode {
    /// Nothing is the tool named.
    fn not_found() -> Self;
    /// No grant of the door's action reaches it.
    fn forbidden() -> Self;
    /// The tool's image declares no such door: `NoFrontend`, or
    /// `NoApi`.
    fn undeclared() -> Self;
    /// No answer, and why.
    fn error(error: Error) -> Self;
}

/// [`Answer`] for a response frame of the SDK's, by its variants.
macro_rules! answer {
    ($frame:ty, $undeclared:ident) => {
        impl Answer for $frame {
            fn not_found() -> Self {
                Self::NotFound
            }

            fn forbidden() -> Self {
                Self::Forbidden
            }

            fn undeclared() -> Self {
                Self::$undeclared
            }

            fn error(error: Error) -> Self {
                Self::Error(error)
            }
        }
    };
}

answer!(diverge_sdk::daemon::endpoints::standalone_tools::frontend::server::response::Frame, NoFrontend);
answer!(diverge_sdk::daemon::endpoints::standalone_tools::api::server::response::Frame, NoApi);
answer!(diverge_sdk::daemon::endpoints::dependency_tools::frontend::server::response::Frame, NoFrontend);
answer!(diverge_sdk::daemon::endpoints::dependency_tools::api::server::response::Frame, NoApi);

/// Who is calling, as the request through a door is stamped with it
/// and a connect's exchanges are: the daemon's own word for whoever
/// put the request. The rules:
///
/// 1. **An account, or an administrator.** A `client`: the identity
///    its handshake established, and a scope id minted here, for this
///    one scope.
/// 2. **An agent's container.** The `agent` its own MCP calls are
///    stamped with, taken from its run: the same caller, with no
///    scope, so a tool knows the agent as one whichever way it comes.
/// 3. **A tool's container.** A `tool`: the tool as its run attests
///    it, and a scope id minted here, as a client's is.
/// 4. **A container that has no run** — its request outlived its
///    container, or came before its start was over — is no caller:
///    the reason, which the handler answers as its `Error`.
pub async fn caller(daemon: &Daemon, who: &Who, standing: &Standing) -> Result<Caller, String> {
    match who {
        // Rule 1.
        Who::Account(_) | Who::Admin(_) => match &standing.identity {
            Some(identity) => Ok(Caller::Client { identity: identity.clone(), scope: uuid::Uuid::new_v4().to_string() }),
            None => Err("the requester has no identity".to_string()),
        },
        Who::Container(declared) => match declared.container {
            // Rule 2.
            Container::Agent(id) => match daemon.live.agent_run(id).await {
                Some(run) => Ok(run.attestation.clone()),
                // Rule 4.
                None => Err("the calling agent's container is not running".to_string()),
            },
            // Rule 3.
            Container::Tool(tool) => match daemon.live.tool_run(tool.key()).await {
                Some(run) => Ok(Caller::Tool { tool: run.attestation.clone(), scope: uuid::Uuid::new_v4().to_string() }),
                // Rule 4.
                None => Err("the calling tool's container is not running".to_string()),
            },
        },
    }
}

/// Whether the image's `/.diverge/DIVERGE.md` declares `door`: `Ok(true)`
/// where it says so, `Ok(false)` where it does not or the image ships
/// none, and `Err` with the reason where the document could not be
/// read.
pub async fn declares(daemon: &Arc<Daemon>, image: &Image, door: Door) -> Result<Result<bool, String>, store::Error> {
    Ok(match images::info(daemon, image).await? {
        images::Read::Info(document) => Ok(door.declared_by(&document)),
        images::Read::Missing => Ok(false),
        images::Read::TooLarge => Err("the image's document could not be read: it is larger than the limit".to_string()),
        images::Read::NoSource => Err("the image's document could not be read: no source holds the image".to_string()),
        images::Read::Error(error) => Err(format!("the image's document could not be read: {}", error.0)),
    })
}

/// Send `head` through `door` to the program in the container, the body the
/// client's channels give, and send every frame of the answer, as the
/// family's answer `R`.
///
/// The rules:
///
/// 1. Each frame of the answer goes on as it comes, paced by the
///    client's reading.
/// 2. `Body` and `End` are served as they arrive, beside the answer:
///    each piece is handed to the request's body, one ahead, and its
///    channel finished once taken; `End` ends the body. A channel that
///    does not decode is finished and does nothing.
/// 3. The scope ends at the answer's end, at the client's `Cancel`, or
///    at the client gone; a stream that breaks is the error, after what
///    was sent. Its end drops the request.
/// 4. A body the provider never asked for is taken back at the end.
/// 5. The [`Opened`] is held for the whole of it: the request is a use
///    of the container, which holds it up.
/// 6. The request is stamped with `caller`: every header named
///    [`KEY`](mcp::KEY), in any case, is removed, and one is set to
///    [`mcp::header`] of the caller — as `_meta` is stamped on an MCP
///    request, nothing under the key the client sent is kept.
///
/// Every family's channels, and both doors', are the same bytes, read
/// here as the standalone tools' frontend's.
pub async fn door<R>(scope: &ScopeHandle, opened: &Opened, mut head: frontend::request::Head, caller: &Caller, door: Door) -> Result<(), String>
where
    R: From<frontend::output::Owned> + Encode,
{
    // Rule 6.
    head.headers.retain(|(name, _)| !name.eq_ignore_ascii_case(mcp::KEY));
    head.headers.push((mcp::KEY.to_string(), mcp::header(caller)));
    let (pieces, taken) = mpsc::channel::<Bytes>(1);
    let body = futures_util::stream::unfold(taken, |mut taken| async move { taken.recv().await.map(|piece| (Ok::<Bytes, Infallible>(piece), taken)) });
    let (mut output, write_id) = opened.door(door, head, body).await?;
    let ended = relay::<R>(scope, &mut output, pieces).await;
    // Rule 4.
    opened.forget(write_id).await;
    ended
}

/// Rules 1 to 3: the answer out, the body in.
async fn relay<R>(scope: &ScopeHandle, output: &mut crate::containers::DoorFrames, pieces: mpsc::Sender<Bytes>) -> Result<(), String>
where
    R: From<frontend::output::Owned> + Encode,
{
    let mut pieces = Some(pieces);
    let mut serving = FuturesUnordered::new();
    loop {
        tokio::select! {
            frame = output.next() => match frame {
                // Rule 1.
                Some(Ok(frame)) => reply::reply(scope, &R::from(frame)).await,
                // Rule 3.
                Some(Err(error)) => return Err(error),
                None => return Ok(()),
            },
            request = scope.recv_channel_request() => {
                // Rule 3: the client gone.
                let Some(bytes) = request else {
                    return Ok(());
                };
                let Ok(ClientFrame::ChannelRequest { channel, payload, .. }) = ClientFrame::decode(&bytes) else {
                    continue;
                };
                match channel_request::Frame::decode(payload) {
                    // Rule 3.
                    Ok(channel_request::Frame::Cancel) => return Ok(()),
                    // Rule 2.
                    Ok(channel_request::Frame::Body(bytes)) => match &pieces {
                        Some(sender) => {
                            let sender = sender.clone();
                            let piece = Bytes::copy_from_slice(bytes);
                            serving.push(async move {
                                let _ = sender.send(piece).await;
                                channel
                            });
                        }
                        None => scope.send_channel_response_finish(channel).await,
                    },
                    Ok(channel_request::Frame::End) => {
                        pieces = None;
                        scope.send_channel_response_finish(channel).await;
                    }
                    Err(_) => scope.send_channel_response_finish(channel).await,
                }
            }
            Some(channel) = serving.next(), if !serving.is_empty() => scope.send_channel_response_finish(channel).await,
        }
    }
}
