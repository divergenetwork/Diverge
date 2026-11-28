//! A tool attached or detached while agents run: their served sets
//! kept in step, and each told its tool list changed.

use std::collections::HashMap;
use std::sync::Arc;

use super::{Guard, ToolKey, serve_name, serve_name_connected};
use crate::daemon::Daemon;
use crate::store::connected_standalone_tools::ConnectedTool;
use crate::store::tools::Tool;
use crate::store::AgentId;

/// The tool is attached to every one of `agents` now — directly, or
/// through their template: each one running has it served from this
/// instant, under its name, and hears its tool list changed; one
/// served it already keeps it as it was. One that holds its tools now
/// — in setup, a message on its way, or active — holds the
/// tool from this instant, as it holds every tool it is served: see
/// [`Holding`](super::Holding). The tool's container is
/// deployed at the agent's next use of it, or before the next message
/// is delivered to it, not here.
pub async fn attached(daemon: &Arc<Daemon>, tool: &Tool, agents: &[AgentId]) {
    let key = ToolKey::Record(tool.id);
    for agent in agents {
        if let Some(run) = daemon.live.agent_run(*agent).await {
            let name = serve_name(tool);
            let mut holding = run.holding.held.lock().await;
            run.served.lock().await.insert(tool.clone(), &name);
            held(daemon, &mut holding, key).await;
        }
    }
}

/// The agent whose `holding` this is was just served the tool: held
/// from now, where the agent holds its tools and does not hold it
/// already.
async fn held(daemon: &Daemon, holding: &mut Option<HashMap<ToolKey, Guard>>, key: ToolKey) {
    if let Some(held) = holding.as_mut()
        && !held.contains_key(&key)
    {
        held.insert(key, daemon.live.hold_agent(key).await);
    }
}

/// The tool is detached from every one of `agents` one way — directly,
/// or through their template — and `keeps` are the agents that have it
/// still the other way, by the records as of the detach: each of the
/// rest that is running serves it no more from this instant and hears
/// its tool list changed. A standalone tool and a connected one alike,
/// by its run's key. An exchange in flight to the tool is
/// answered still, holding the tool's run up until it is; an agent
/// that held its tools lets its hold on the tool go; nothing of
/// the run is stopped here.
pub async fn detached(daemon: &Arc<Daemon>, tool: ToolKey, agents: &[AgentId], keeps: &[AgentId]) {
    for agent in agents {
        if keeps.contains(agent) {
            continue;
        }
        if let Some(run) = daemon.live.agent_run(*agent).await {
            let let_go = {
                let mut holding = run.holding.held.lock().await;
                run.served.lock().await.remove(tool);
                holding.as_mut().and_then(|held| held.remove(&tool))
            };
            if let_go.is_some() {
                drop(let_go);
                if let Some(run) = daemon.live.tool_run(tool).await {
                    run.touch();
                }
                daemon.live.release_holds(tool).await;
            }
        }
    }
}

/// The connected tool is attached to every one of `agents` now, as
/// [`attached`] states for a standalone tool: served from this instant
/// under its name, held by an agent that holds its tools, its connect
/// opened at the agent's next use of it, or before the next message
/// is delivered to it.
pub async fn attached_connected(daemon: &Arc<Daemon>, tool: &ConnectedTool, agents: &[AgentId]) {
    for agent in agents {
        if let Some(run) = daemon.live.agent_run(*agent).await {
            let name = serve_name_connected(tool);
            let mut holding = run.holding.held.lock().await;
            run.served.lock().await.insert_connected(tool.clone(), &name);
            held(daemon, &mut holding, ToolKey::Connected(tool.id)).await;
        }
    }
}
