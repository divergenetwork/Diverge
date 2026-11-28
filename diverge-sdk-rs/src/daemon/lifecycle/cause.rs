//! What brought a run's container up.

use serde::{Deserialize, Serialize};

/// What deployed a run, as its [`Deployed`](super::Deployed) says.
/// Exactly one per deploy: the one thing that found the container
/// down and started it. What uses the container after that is not
/// told here. Snake case on the wire.
///
/// An agent's run is deployed by a [`Message`](Self::Message), a
/// [`Connect`](Self::Connect), a [`DependencyTool`](Self::DependencyTool)
/// or one of the seven requests into its container. A tool's run is
/// deployed by a [`Registration`](Self::Registration), an
/// [`AgentActive`](Self::AgentActive), an
/// [`Attachment`](Self::Attachment), a [`Restart`](Self::Restart), a
/// [`Connect`](Self::Connect) or one of the seven requests into its
/// container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cause {
    /// A message for the agent, which starts a loop. An agent's alone.
    Message,
    /// A connect scope: on an agent's and on a standalone tool's, the
    /// scope's opening; and, of a tool an agent is served, an exchange
    /// run as the agent on an agent's connect scope that found the
    /// tool down.
    Connect,
    /// An agent's container started and its program declared the tool
    /// at its registration. A dependency tool's alone.
    Registration,
    /// An agent the tool is attached to began a loop. A tool's alone.
    AgentActive,
    /// An agent came to be attached to an eager tool. A tool's alone.
    Attachment,
    /// An eager tool that is still held was started again: after its
    /// container ended, or at the daemon's start. A tool's alone.
    Restart,
    /// A dependency tool of the agent's that mounts the agent's files
    /// was deployed while the agent's container was down. An agent's
    /// alone.
    DependencyTool,
    /// A download of one of the container's files.
    Download,
    /// An upload of a file into the container.
    Upload,
    /// A transfer of a file, out of the container or into it.
    Transfer,
    /// A watch of the container's file tree.
    Filetree,
    /// A command run in the container.
    Exec,
    /// A terminal opened in the container.
    Terminal,
    /// A request to the frontend the container's program serves.
    Frontend,
}
