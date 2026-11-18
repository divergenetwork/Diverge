//! The agents a grant lists.

use serde::{Deserialize, Serialize};

use super::Key;

/// Exactly the agents listed, as a grant's `within` names them: the
/// object `{"agents":[…]}`, one member and no other, so that it
/// decodes as neither the word `"any"` nor a tags test. Each is a [`Key`]: the template and index that are one agent's on the daemon holding it.
/// It reaches those and no other, whatever their tags, judged at every
/// request; one that does not exist, or no longer does, is reached by
/// nothing. An empty list reaches nothing. See
/// [`Reach`](crate::shared::permission::Reach).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "AgentsListed")]
#[serde(deny_unknown_fields)]
pub struct Listed {
    /// The agents reached.
    pub agents: Vec<Key>,
}
