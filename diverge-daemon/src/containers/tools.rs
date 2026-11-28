//! One container per tool: deployed for the first thing that uses it,
//! held up as [`idle_tool`](super::idle_tool) states, and undeployed by
//! its idle clock `idle_seconds` after the last hold left and its last
//! use.

use std::collections::HashSet;
use std::sync::Arc;

use diverge_sdk::daemon::lifecycle::Cause;

use super::deploy::Mounted;
use super::{ConnectionId, StartError, ToolKey, ToolRun, deploy, eager, start};
use crate::daemon::Daemon;
use crate::store::connected_standalone_tools::ConnectedTool;
use crate::store::tools::Tool;
use crate::store::{DependencyToolId, agents, dependency_tools};

/// The tool's run: deployed now if it was not running, `cause` being
/// what deployed it, and touched if it was. Nothing is held by this:
/// whoever uses the run takes the hold its use is — an
/// [`Inflight`](super::Inflight) guard, a file use, a [`connect`]. An
/// eager tool is told to read its attachments anew, so that one found
/// attached is kept.
pub async fn use_tool(daemon: &Arc<Daemon>, tool: &Tool, cause: Cause) -> Result<Arc<ToolRun>, StartError> {
    let run = match daemon.live.tool_run(ToolKey::Record(tool.id)).await {
        Some(run) => {
            run.touch();
            run
        }
        None => start::tool(daemon, tool, cause).await?,
    };
    if run.eager {
        eager::held(daemon, run.id, cause).await;
    }
    Ok(run)
}

/// The dependency tool's run: deployed now if it was not running,
/// `cause` being what deployed it, and touched if it was. Nothing is
/// held by this, as for [`use_tool`]. The rules, by what the tool is:
///
/// 1. **Running.** The run live under its row is the answer, touched.
/// 2. **Gone.** A row that is not on record is
///    [`StartError::Gone`], and so is a tool of one agent whose agent
///    is not on record.
/// 3. **Eager, with an agent attached.** It is kept by its
///    supervisor, which starts it: its attachments are read anew, and
///    its next start is waited for.
/// 4. **Eager with none attached, or lazy of the `template` scope.**
///    Deployed alone, with no mount.
/// 5. **Lazy of the `instance` scope, mounting nothing.** Deployed
///    alone, its agent left as it is.
/// 6. **Lazy of the `instance` scope, mounting its agent's files.**
///    Its agent's container is what its mounts are served from, so
///    the agent is started first if it is down — the ordinary start,
///    whose registration deploys every tool the agent's program
///    declares, this one among them if it still does. Then, in order:
///    the run live under the row is the answer; an agent that no
///    longer declares the tool has it no more, and the answer is
///    [`StartError::Gone`]; otherwise — the agent was up already and
///    the tool had gone idle — it is deployed now, mounting from the
///    agent's run. Its run holds a use on its agent's run for as long
///    as it is up.
pub async fn use_dependency(daemon: &Arc<Daemon>, id: DependencyToolId, cause: Cause) -> Result<Arc<ToolRun>, StartError> {
    let key = ToolKey::Dependency(id);
    // Rule 1.
    if let Some(run) = daemon.live.tool_run(key).await {
        run.touch();
        return Ok(run);
    }
    // Rule 2.
    let tool = {
        let mut conn = daemon.store.acquire().await?;
        dependency_tools::by_id(&mut conn, id, false).await?
    };
    let Some(tool) = tool else {
        return Err(StartError::Gone);
    };
    if tool.is_eager() {
        eager::held(daemon, key, cause).await;
        // Rule 3.
        if daemon.live.eager_state(key).await.is_some() {
            eager::up(daemon, key).await.map_err(StartError::Run)?;
            return daemon.live.tool_run(key).await.ok_or_else(|| StartError::Run("the eager tool is not running now".to_string()));
        }
        // Rule 4.
        return deploy::tool(daemon, &tool, None, cause).await;
    }
    // Rules 4 and 5.
    if !tool.declared.mounts() {
        return deploy::tool(daemon, &tool, None, cause).await;
    }
    // Rule 6.
    let Some(owner) = tool.owner_agent else {
        return Err(StartError::Gone);
    };
    let agent = {
        let mut conn = daemon.store.acquire().await?;
        agents::by_id(&mut conn, owner, false).await?
    };
    let Some(agent) = agent else {
        return Err(StartError::Gone);
    };
    let agent = start::agent_for(Arc::clone(daemon), agent, Cause::DependencyTool).await?;
    if let Some(run) = daemon.live.tool_run(key).await {
        run.touch();
        return Ok(run);
    }
    let declared = {
        let mut conn = daemon.store.acquire().await?;
        match dependency_tools::by_id(&mut conn, id, false).await? {
            Some(tool) => dependency_tools::attachments::attached(&mut conn, id).await?.iter().any(|attachment| attachment.agent == owner).then_some(tool),
            None => None,
        }
    };
    let Some(tool) = declared else {
        return Err(StartError::Gone);
    };
    deploy::tool(daemon, &tool, Some(Mounted::of(&agent)), cause).await
}

/// The connected tool's run — this daemon's connect on the other
/// daemon's tool: opened now if it was not held, `cause` being what
/// opened it, and touched if it was. Nothing is held by this, as for
/// [`use_tool`]; a connected tool is never an eager tool here, and
/// never served onward to another daemon.
pub async fn use_connected(daemon: &Arc<Daemon>, tool: &ConnectedTool, cause: Cause) -> Result<Arc<ToolRun>, StartError> {
    match daemon.live.tool_run(ToolKey::Connected(tool.id)).await {
        Some(run) => {
            run.touch();
            Ok(run)
        }
        None => start::connected(daemon, tool, cause).await,
    }
}

/// A connect scope of another daemon's holds the run from now on,
/// until the [`Hold`] handed back is let go — [`Hold::disconnect`] on
/// the scope's end, or the guard dropped however else it ends: the run
/// is told it is held.
pub async fn connect(run: &Arc<ToolRun>, connection: ConnectionId) -> Hold {
    let mut connections = run.connections.lock().await;
    connections.insert(connection);
    held(run, &connections);
    Hold { run: Arc::clone(run), connection: Some(connection) }
}

/// One connect scope's hold on a run, from [`connect`]. A drop cannot
/// wait on the run's lock, so a hold dropped [`disconnect`]s in a task
/// of its own.
pub struct Hold {
    /// The run held.
    run: Arc<ToolRun>,
    /// The scope's number, until it is disconnected.
    connection: Option<ConnectionId>,
}

impl Hold {
    /// The connect scope is over: [`disconnect`], waited for.
    pub async fn disconnect(mut self) {
        if let Some(connection) = self.connection {
            disconnect(&self.run, connection).await;
            self.connection = None;
        }
    }
}

impl Drop for Hold {
    fn drop(&mut self) {
        let Some(connection) = self.connection.take() else {
            return;
        };
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            let run = Arc::clone(&self.run);
            runtime.spawn(async move { disconnect(&run, connection).await });
        }
    }
}

/// The connect scope is over: the run is told whether any connect
/// scope holds it still. A tool whose last scope left is the idle
/// clock's: stopped `idle_seconds` after, unless something holds or
/// uses it meanwhile.
pub async fn disconnect(run: &ToolRun, connection: ConnectionId) {
    let mut connections = run.connections.lock().await;
    connections.remove(&connection);
    held(run, &connections);
}

/// The word that a connect scope holds the run, or none does, kept in
/// step with `connections` under its lock.
fn held(run: &ToolRun, connections: &HashSet<ConnectionId>) {
    run.held.send_replace(!connections.is_empty());
}
