//! Whose a dependency tool is: one agent's, or everyone's.

use serde::{Deserialize, Serialize};

use super::Agent;

/// Whose a dependency tool is, as its image's `scope` names it: the
/// one agent it is for, where the `scope` is `instance`, or nobody in
/// particular, every agent that declares it being attached to the
/// one. The agent is named either way, by name or once and for all:
/// see [`Agent`]. Externally tagged on the wire, snake case:
/// `{"agent":{"name":…}}` or `{"agent":{"template":…,"index":…}}`, or
/// the string `"global"`. The daemon answers the once-and-for-all
/// form, as [`key::Owner`](crate::daemon::key::Owner).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Owner {
    /// The one agent: a tool's whose template's `scope` is
    /// `instance`.
    Agent(Agent),
    /// Every agent that declares the template: a tool's whose
    /// template's `scope` is `global`.
    Global,
}
