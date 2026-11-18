//! How far a dependency tool is shared.

use serde::{Deserialize, Serialize};

/// What a dependency tool is one of: one per parent agent, one per
/// parent agent template, or one for every agent that declares it,
/// whatever template they are made from. The scope names the tool —
/// the same template declared under the same scope key is the same
/// tool, joined and not made again — and names its database scope
/// with it, so what a tool holds is shared exactly as far as the tool
/// is. Both modes say one: see
/// [`Mode`](super::Mode). Snake case on the wire: `"instance"`,
/// `"template"`, `"global"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    /// One tool per parent agent: the tool deployed for one agent is
    /// its own, with a database of its own, and two agents' tools of
    /// this template never see each other's. The one scope under
    /// which a lazy tool mounts the agent's files.
    Instance,
    /// One tool per parent agent template: every agent made from the
    /// same agent template that declares this template is attached
    /// to one tool, which holds one database, so what one writes the
    /// others read.
    Template,
    /// One tool for the template, full stop: every agent that
    /// declares it, from whatever agent template, is attached to the
    /// one tool, and its database is one for all of them.
    Global,
}
