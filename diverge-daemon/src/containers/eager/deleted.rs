//! What a delete does to the eager tools it reaches.

use std::sync::Arc;

use diverge_sdk::daemon::lifecycle::Cause;

use super::{held, retire};
use crate::containers::ToolKey;
use crate::daemon::Daemon;
use crate::store::dependency_tools::DependencyTool;
use crate::store::{DependencyToolId, ToolId};

/// The agent is deleted: `removed` were the dependency tools it
/// owned, whose rows went with it — each one's run stopped, and its
/// keeping ended where it was an eager tool — `left` the dependency
/// tools it was attached to and did not own, and `detached` the tools
/// on record it was attached to, each told to read its attachments
/// anew, which leaves an eager tool to its idle clock where the agent
/// was the last attached to it.
pub async fn agent_deleted(daemon: &Arc<Daemon>, removed: Vec<DependencyTool>, left: Vec<DependencyToolId>, detached: Vec<ToolId>) {
    template_deleted(daemon, removed).await;
    for id in left {
        held(daemon, ToolKey::Dependency(id), Cause::Attachment).await;
    }
    for id in detached {
        held(daemon, ToolKey::Record(id), Cause::Attachment).await;
    }
}

/// `removed` were the dependency tools whose rows went — an agent's
/// own with the agent, or every one of a dependency template with its
/// compartment: each is retired — its run, if one is up, stopped,
/// since its row is gone, and its keeping ended where it was an eager
/// tool.
pub async fn template_deleted(daemon: &Arc<Daemon>, removed: Vec<DependencyTool>) {
    for tool in removed {
        retire(daemon, ToolKey::Dependency(tool.id)).await;
    }
}

/// The tool on record is deleted: its run, if one is up, is stopped,
/// and its keeping ended if it was an eager tool kept.
pub async fn tool_deleted(daemon: &Arc<Daemon>, id: ToolId) {
    retire(daemon, ToolKey::Record(id)).await;
}
