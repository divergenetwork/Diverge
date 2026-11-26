//! How far a dependency tool is shared.

use serde::{Deserialize, Serialize};

/// What a dependency tool is one of: one per parent agent, or one for
/// every agent that declares it, whatever template they are made
/// from. A [`DependencyTool`](super::DependencyTool)'s document states
/// one, and no declaration says another: how far a tool is shared is
/// what its program is built for. A
/// [`StandaloneTool`](super::StandaloneTool)'s document has none, and
/// an agent's has none.
///
/// The scope names the tool: the same declaration under the same
/// scope key is the same tool, joined and not made again. It does not
/// name what the tool's databases are — those are the same three for
/// every container, see
/// [`Database`](crate::container_proxy::inside::Database) — but for
/// one thing: a global tool runs for no one agent, and so has no
/// `instance` database. Snake case on the wire: `"instance"`,
/// `"global"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    /// One tool per parent agent: the tool deployed for one agent is
    /// its own, and two agents' tools of this image are two
    /// containers. The one scope under which a lazy tool mounts the
    /// agent's files.
    Instance,
    /// One tool, full stop: every agent that declares it, from
    /// whatever agent template, is attached to the one tool.
    Global,
}
