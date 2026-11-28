//! What brought a run's container up.

use serde::{Deserialize, Serialize};

/// What deployed a run, as its [`Deployed`](super::Deployed) says.
/// Exactly one per deploy: the one thing that found the container
/// down and started it. What uses the container after that is not
/// told here. Snake case on the wire.
///
/// An agent's run is deployed by a [`Message`](Self::Message), a
/// [`Connect`](Self::Connect), a [`DependencyTool`](Self::DependencyTool)
/// or one of the six requests into its container: a download, an
/// upload, a transfer, a file tree, a command, a terminal. A tool's run is
/// deployed by a [`Registration`](Self::Registration), an
/// [`AgentDeployed`](Self::AgentDeployed), an
/// [`AgentActive`](Self::AgentActive), an
/// [`Attachment`](Self::Attachment), a [`Restart`](Self::Restart), a
/// [`Connect`](Self::Connect) or one of the eight requests into its
/// container: those six, a [`Frontend`](Self::Frontend) request and
/// an [`Api`](Self::Api) request.
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
    /// An agent the tool is served to was deployed: every tool an
    /// agent is served is deployed with it. A standalone or a
    /// connected tool's; a dependency tool the agent's program
    /// declared is a [`Registration`](Self::Registration).
    AgentDeployed,
    /// A tool an agent is served had gone down while the agent's
    /// container stayed up, and was brought back for the agent: before
    /// a message was delivered to it, or by an exchange of its own
    /// container that found the tool down. A tool's alone.
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
    /// A request to the frontend the tool's program serves. A tool's
    /// alone: an agent's program serves none.
    Frontend,
    /// A request to the API the tool's program serves. A tool's alone.
    Api,
}
