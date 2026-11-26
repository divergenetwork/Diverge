//! What a delete does to the eager tools it reaches.

use std::sync::Arc;

use super::{held, retire};
use crate::containers::lifecycle::End;
use crate::containers::{ToolKey, stop};
use crate::daemon::Daemon;
use crate::store::dependency_tools::DependencyTool;
use crate::store::{DependencyToolId, ToolId};

/// The agent is deleted: `removed` were the dependency tools it
/// owned, whose rows went with it — each eager tool among them retired,
/// its run with it — `left` the dependency tools it was attached to
/// and did not own, and `detached` the tools on record it was
/// attached to, each told its hold changed, which starts an eager tool's
/// clock where the agent was the last to hold it.
pub async fn agent_deleted(daemon: &Arc<Daemon>, removed: Vec<DependencyTool>, left: Vec<DependencyToolId>, detached: Vec<ToolId>) {
    template_deleted(daemon, removed).await;
    for id in left {
        held(daemon, ToolKey::Dependency(id)).await;
    }
    for id in detached {
        held(daemon, ToolKey::Record(id)).await;
    }
}

/// `removed` were the dependency tools whose rows went — an agent's
/// own with the agent, or every one of a dependency template with its
/// compartment: each eager tool among them is retired, its run with
/// it; a lazy tool among them has no keeping to end.
pub async fn template_deleted(daemon: &Arc<Daemon>, removed: Vec<DependencyTool>) {
    for tool in removed {
        if tool.is_eager() {
            retire(daemon, ToolKey::Dependency(tool.id)).await;
        } else {
            // A lazy tool has no keeping to end: its run, if one is
            // up, is stopped, since its row is gone.
            if let Some(run) = daemon.live.tool_run(ToolKey::Dependency(tool.id)).await {
                stop::stop_dependencies(daemon, vec![run], End::Expected).await;
            }
        }
    }
}

/// The tool on record is deleted: its keeping, if it was an eager tool
/// kept — by a connect scope, or pending its clock, since a delete is
/// refused while any agent is attached — is ended, its run with it.
pub async fn tool_deleted(daemon: &Arc<Daemon>, id: ToolId) {
    retire(daemon, ToolKey::Record(id)).await;
}
