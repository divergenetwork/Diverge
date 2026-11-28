//! Every tool an agent is served, deployed.

use std::sync::Arc;

use diverge_sdk::daemon::lifecycle::Cause;
use futures_util::future;
use tokio::sync::Mutex;

use super::{Mounted, tool};
use crate::containers::mcp::{Entry, Served};
use crate::containers::{StartError, tools};
use crate::daemon::Daemon;
use crate::store::DependencyToolId;
use crate::store::connected_standalone_tools::ConnectedTool;
use crate::store::dependency_tools;
use crate::store::tools::Tool;

/// What deploys a tool an agent is served, by which kind of tool it
/// is.
#[derive(Debug, Clone, Copy)]
pub struct Causes {
    /// What deploys a standalone or a connected tool attached to the
    /// agent.
    pub attached: Cause,
    /// What deploys a dependency tool the agent's program declared.
    pub declared: Cause,
}

impl Causes {
    /// An agent's start: a tool attached to it is deployed because the
    /// agent was, and a dependency tool because the registration
    /// declared it.
    pub const SETUP: Causes = Causes { attached: Cause::AgentDeployed, declared: Cause::Registration };
    /// A message for an agent that was up already: whichever tool it
    /// is, it is brought back for the loop the message starts.
    pub const MESSAGE: Causes = Causes { attached: Cause::AgentActive, declared: Cause::AgentActive };
}

/// One served tool that is down, as it is deployed.
enum Down {
    /// A standalone tool attached to the agent.
    Attached(Tool),
    /// A connected tool attached to the agent.
    Connected(ConnectedTool),
    /// A dependency tool, of whichever mode and scope.
    Dependency(DependencyToolId),
}

/// Every tool in `served` that is down, deployed: what an agent's
/// start does once its registration is over, and what a message does
/// before it is delivered to an agent that was up already. `mounted`
/// is the agent's container, which a tool that mounts the agent's
/// files is deployed against. The rules:
///
/// 1. **Every kind.** A standalone tool attached to the agent or to
///    its template, a connected tool attached the same way, and a
///    dependency tool its program declared — lazy or eager, of either
///    scope. None is left for its first use.
/// 2. **Only what is down.** A tool whose run is live is left as it
///    is, and nothing of it is touched.
/// 3. **All at once, all waited for.** Every one that is down is
///    deployed at the same time, each by the start its kind has — a
///    start in a task of its own, one at a time per tool — and every
///    one is waited for before this answers.
/// 4. **The first that fails is the error**, in the order the agent
///    is served them: the name the agent is served it under, and the
///    reason. Every other one that came up is left up, to its own
///    holds and its idle clock: nothing is stopped here.
/// 5. **No start of the agent's is waited for.** A tool that mounts
///    the agent's files is deployed against `mounted` directly, and
///    never by the way that starts its agent first —
///    [`use_dependency`](tools::use_dependency) — so this may run
///    inside the agent's own start, under its start lock, and wait
///    on nothing that waits on that lock. A tool's start waits for no
///    agent's otherwise.
/// 6. **Nothing is held by this.** The agent holds every tool it is
///    served while this runs — it is in setup, or a message is on its
///    way — as [`Holding`](crate::containers::Holding) states; that,
///    and not this, is what keeps what is deployed here up.
pub async fn served(daemon: &Arc<Daemon>, served: &Mutex<Served>, mounted: &Mounted, causes: Causes) -> Result<(), (String, String)> {
    // Rule 2: read under the served lock, and acted on after it.
    let mut down = Vec::new();
    {
        let set = served.lock().await;
        for key in set.keys() {
            let one = match set.entry(key) {
                Some(Entry::Attached { tool }) => Down::Attached(tool.clone()),
                Some(Entry::Connected { tool }) => Down::Connected(tool.clone()),
                Some(Entry::Own { id, .. }) | Some(Entry::Eager { id, .. }) | Some(Entry::Shared { id, .. }) => Down::Dependency(*id),
                None => continue,
            };
            down.push((key, set.prefix(key).unwrap_or_default(), one));
        }
    }
    let mut starting = Vec::new();
    for (key, name, one) in down {
        if daemon.live.tool_run(key).await.is_none() {
            starting.push((name, one));
        }
    }
    if starting.is_empty() {
        return Ok(());
    }
    // Rule 3.
    let started = future::join_all(starting.into_iter().map(|(name, one)| async move {
        let outcome = match one {
            Down::Attached(record) => tools::use_tool(daemon, &record, causes.attached).await.map(|_| ()),
            Down::Connected(record) => tools::use_connected(daemon, &record, causes.attached).await.map(|_| ()),
            Down::Dependency(id) => dependency(daemon, id, mounted, causes.declared).await,
        };
        (name, outcome)
    }))
    .await;
    // What the notifications relay keeps in step with: a served tool's
    // run started.
    served.lock().await.started();
    // Rule 4.
    match started.into_iter().find_map(|(name, outcome)| outcome.err().map(|error| (name, error.to_string()))) {
        Some(failed) => Err(failed),
        None => Ok(()),
    }
}

/// Rule 5 of [`served`]: one dependency tool deployed — against the
/// agent's container where it mounts the agent's files, and as any
/// other use of it deploys it where it mounts nothing.
async fn dependency(daemon: &Arc<Daemon>, id: DependencyToolId, mounted: &Mounted, cause: Cause) -> Result<(), StartError> {
    let row = {
        let mut conn = daemon.store.acquire().await?;
        dependency_tools::by_id(&mut conn, id, false).await?
    };
    let Some(row) = row else {
        return Err(StartError::Gone);
    };
    if row.declared.mounts() {
        tool(daemon, &row, Some(mounted.clone()), cause).await.map(|_| ())
    } else {
        tools::use_dependency(daemon, id, cause).await.map(|_| ())
    }
}
