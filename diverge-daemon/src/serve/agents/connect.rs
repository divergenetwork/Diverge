//! MCP as an agent: the agent's container deployed and held up for
//! the scope's life, and the agent's exchanges answered on the
//! scope's channels as its own container's are.

use std::sync::Arc;

use diverge_sdk::daemon::endpoints::agents::connect::client::channel_request;
use diverge_sdk::daemon::endpoints::agents::connect::client::request;
use diverge_sdk::daemon::endpoints::agents::connect::server::response::Frame;
use diverge_sdk::daemon::grant::agents::Over;
use diverge_sdk::daemon::lifecycle::Cause;
use diverge_sdk::shared::mcp;
use diverge_sdk::wire::decode::Decode as _;
use diverge_sdk::wire::encode::{Encode, Writer};
use diverge_sdk::wire::frame::client::ClientFrame;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;
use futures_util::StreamExt as _;
use tokio::task::JoinSet;

use super::state;
use crate::containers::{self, AgentRun};
use crate::daemon::Daemon;
use crate::judge::{self, Standing, Who};
use crate::serve::reply;
use crate::store::{self, agents};

/// Answer the connect, serve its channels until the client
/// disconnects, its connection ends or the agent's run ends, then
/// finish the scope.
pub async fn handle(scope: ScopeHandle, frame: request::Frame, who: Who, daemon: &Arc<Daemon>) {
    let scope = Arc::new(scope);
    if let Err(error) = serve(&scope, frame, who, daemon).await {
        reply::reply(&scope, &Frame::Error(reply::failure(&error))).await;
    }
    scope.send_response_finish().await;
}

/// The rules:
///
/// 1. **Judged first.** `Forbidden` with no `connect` grant at all;
///    `NotFound` for no such agent; `Forbidden` for an agent the
///    grants do not reach. No account is asked for: what is stamped
///    on every exchange is the agent, not the client.
/// 2. **Deployed, and held.** The agent's run, started if it is not
///    up — the ordinary start, which deploys every tool it is served,
///    its cause `connect` — and waited for; one that will not start is
///    the connect's `Error`, and the scope finishes. From here the
///    scope holds the run up, as a use does, until it ends: the idle
///    clock does not run, and counts from the scope's end. The hold
///    makes the agent neither active nor busy.
/// 3. **Then connected.** Exactly one `Connected`.
/// 4. **Every channel is one exchange,** answered on a task of its own
///    (see [`answer`]); a channel request that does not decode is
///    finished with nothing.
/// 5. **Three ends.** A disconnect, the client's connection ending,
///    or the agent's run ending — however it ends: stopped, deleted,
///    its container gone — ends the scope, and every exchange still in
///    flight with it. A run that ended is not started again under the
///    scope.
async fn serve(scope: &Arc<ScopeHandle>, frame: request::Frame, who: Who, daemon: &Arc<Daemon>) -> Result<(), store::Error> {
    let mut conn = daemon.store.acquire().await?;
    // Rule 1.
    let Some(standing) = Standing::of(&mut conn, who).await? else {
        reply::reply(scope, &Frame::Forbidden).await;
        return Ok(());
    };
    if !judge::agents::holds(&standing, Over::Connect) {
        reply::reply(scope, &Frame::Forbidden).await;
        return Ok(());
    }
    let Some(agent) = agents::by_reference(&mut conn, &frame.agent, false).await? else {
        reply::reply(scope, &Frame::NotFound).await;
        return Ok(());
    };
    drop(conn);
    if !judge::agents::over(&standing, Over::Connect, &agent, state(daemon, agent.id).await) {
        reply::reply(scope, &Frame::Forbidden).await;
        return Ok(());
    }
    scope.allowed().await;
    // Rule 2.
    let run = match containers::agent(daemon, &agent, Cause::Connect).await {
        Ok(run) => run,
        Err(error) => {
            reply::reply(scope, &Frame::Error(reply::failure(&error))).await;
            return Ok(());
        }
    };
    let held = run.file_use();
    // Rule 3.
    reply::reply(scope, &Frame::Connected).await;
    // Rules 4 and 5.
    let mut tasks = JoinSet::new();
    loop {
        tokio::select! {
            () = run.gone() => break,
            request = scope.recv_channel_request() => {
                let Some(bytes) = request else {
                    break;
                };
                let Ok(ClientFrame::ChannelRequest { channel, payload, .. }) = ClientFrame::decode(&bytes) else {
                    continue;
                };
                match channel_request::Frame::decode(payload) {
                    Ok(channel_request::Frame::Disconnect) => break,
                    Ok(ask) => {
                        tasks.spawn(answer(Arc::clone(daemon), Arc::clone(scope), Arc::clone(&run), channel, ask));
                    }
                    Err(_) => scope.send_channel_response_finish(channel).await,
                }
            }
            // Every answered exchange reaped as it ends, so the set
            // holds only those in flight.
            Some(_) = tasks.join_next(), if !tasks.is_empty() => {}
        }
    }
    // The scope mints no session of its own: its exchanges are the
    // agent's, on the agent's own session in each tool, which goes
    // with the agent's deletion (`tool_log::forget_agent`), not here.
    tasks.abort_all();
    // Rule 2: the hold let go, which touches the run, so its idle
    // clock counts from here.
    drop(held);
    Ok(())
}

/// One channel, answered as the agent's own container's exchange is,
/// on the run the scope holds. The rules:
///
/// 1. **Nothing of the agent's is counted.** The exchange is not in
///    flight for the agent: it does not make the agent active, does
///    not keep it active, and writes nothing to its log. What holds
///    the agent's run meanwhile is the scope.
/// 2. **The agent's own answer.** The exchange is put to the merge
///    with the agent's own context — its tools by the names it sees
///    them by, its stamp, its sessions, its log entries in each tool's
///    log — exactly as its container's would be.
/// 3. **The tool it reaches is held.** A list, a call or a read is in
///    flight into the tool it reaches until it is answered, as every
///    exchange into a tool is, and a served tool found down is
///    deployed for it, its cause `connect`.
/// 4. **Sent.** One channel response and the finish; the notifications
///    one per notification until their stream ends, then the finish.
async fn answer(daemon: Arc<Daemon>, scope: Arc<ScopeHandle>, run: Arc<AgentRun>, channel: u32, ask: channel_request::Frame) {
    // Rule 3: a served tool is deployed as a connect deploys it, not as
    // the agent active does.
    let context = containers::mcp::Context { cause: Cause::Connect, ..run.context(&daemon) };
    match ask {
        channel_request::Frame::Disconnect => {}
        // Rules 1, 2 and 4.
        channel_request::Frame::McpListTools(request) => {
            let frame = match containers::mcp::list_tools(&context, request.0).await {
                Ok(result) => mcp::list_tools::response::Frame::Result(result),
                Err(error) => mcp::list_tools::response::Frame::Error(error),
            };
            respond(&scope, channel, &frame).await;
        }
        channel_request::Frame::McpListResources(request) => {
            let frame = match containers::mcp::list_resources(&context, request.0).await {
                Ok(result) => mcp::list_resources::response::Frame::Result(result),
                Err(error) => mcp::list_resources::response::Frame::Error(error),
            };
            respond(&scope, channel, &frame).await;
        }
        channel_request::Frame::McpCallTool(request) => {
            let frame = match containers::mcp::call_tool(&context, request.0).await {
                Ok(result) => mcp::call_tool::response::Frame::Result(result),
                Err(error) => mcp::call_tool::response::Frame::Error(error),
            };
            respond(&scope, channel, &frame).await;
        }
        channel_request::Frame::McpReadResource(request) => {
            let frame = match containers::mcp::read_resource(&context, request.0).await {
                Ok(result) => mcp::read_resource::response::Frame::Result(result),
                Err(error) => mcp::read_resource::response::Frame::Error(error),
            };
            respond(&scope, channel, &frame).await;
        }
        channel_request::Frame::McpNotifications(_) => {
            let mut stream = containers::mcp::notifications(&context).await;
            while let Some(item) = stream.next().await {
                let frame = match item {
                    Ok(notification) => mcp::notifications::response::Frame::Notification(notification),
                    Err(error) => mcp::notifications::response::Frame::Error(error),
                };
                let last = matches!(frame, mcp::notifications::response::Frame::Error(_));
                send(&scope, channel, &frame).await;
                if last {
                    break;
                }
            }
            scope.send_channel_response_finish(channel).await;
        }
    }
}

/// One channel response, then the finish: a unary exchange's answer.
async fn respond<F: Encode>(scope: &ScopeHandle, channel: u32, frame: &F) {
    send(scope, channel, frame).await;
    scope.send_channel_response_finish(channel).await;
}

/// One channel response. A frame that will not encode is the one
/// failure with nowhere to go, and is not sent; the finish that every
/// answer ends with is what the client then reads.
async fn send<F: Encode>(scope: &ScopeHandle, channel: u32, frame: &F) {
    let mut bytes = Vec::new();
    if frame.encode(&mut Writer::new(&mut bytes)).is_ok() {
        scope.send_channel_response(channel, &bytes).await;
    }
}
