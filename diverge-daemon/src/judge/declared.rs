//! A container as it is served: the grants its program declared, and
//! which container it is, whose attachments `attached` reaches.

use std::sync::Arc;

use diverge_sdk::daemon::creator::Creator;
use diverge_sdk::daemon::grant::{self, Converted, Grant};

use crate::containers::{Key, ToolKey};
use crate::store::{AgentId, DependencyToolId, ToolId};

/// What a container's program declared at its registration, fixed for
/// the run: the creator its requests make things as, the daemon's
/// grants its declaration converts to, and the actions it holds within
/// `attached`, kind by kind, which no daemon grant can say and which
/// are judged against what is attached to the container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fixed {
    /// What the container's requests make things as: the container
    /// itself, a `Creator::Tool` or a `Creator::Agent`.
    pub creator: Creator,
    /// Every grant a role could hold, in no order.
    pub grants: Vec<Grant>,
    /// The actions held within `attached`, kind by kind, in no order.
    pub attached: grant::Attached,
}

impl Fixed {
    /// From a container's declared permissions, converted.
    pub fn new(creator: Creator, converted: Converted) -> Fixed {
        Fixed {
            creator,
            grants: converted.grants,
            attached: converted.attached,
        }
    }
}

/// Which tool a run is, by its row: a standalone tool or a dependency
/// tool, whose attached agents are read differently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolRef {
    /// A standalone tool: attached to agents directly, and to every
    /// agent of the agent templates it is attached to.
    Standalone(ToolId),
    /// A dependency tool: attached to the agents whose registration
    /// declared its template, and still does.
    Dependency(DependencyToolId),
}

impl ToolRef {
    /// The tool's run's key: what its run, its log and its holds are
    /// kept under.
    pub fn key(self) -> ToolKey {
        match self {
            ToolRef::Standalone(id) => ToolKey::Record(id),
            ToolRef::Dependency(id) => ToolKey::Dependency(id),
        }
    }
}

/// Which container a run is, by its row: the two kinds whose program
/// may open a `/daemon` connection and put requests to the daemon.
/// What tells them apart is what `attached` reaches — see
/// [`Standing`](super::Standing) — and whose log keeps its commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Container {
    /// An agent's container: attached to the tools it is served.
    Agent(AgentId),
    /// A tool's container, standalone or dependency: attached to
    /// agents.
    Tool(ToolRef),
}

impl Container {
    /// The container's run's key.
    pub fn key(self) -> Key {
        match self {
            Container::Agent(id) => Key::Agent(id),
            Container::Tool(tool) => Key::Tool(tool.key()),
        }
    }
}

/// A container, as its `/daemon` connections are served: what its
/// program declared, and which container it is, whose attachments —
/// read fresh for every request — are what `attached` reaches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declared {
    /// What the container's program declared.
    pub fixed: Arc<Fixed>,
    /// The container, by its row: whose attachments `attached`
    /// reaches.
    pub container: Container,
}
