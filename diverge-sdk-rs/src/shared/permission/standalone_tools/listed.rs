//! The standalone tools a grant lists.

use serde::{Deserialize, Serialize};

use super::Key;

/// Exactly the standalone tools listed, as a grant's `within` names them: the
/// object `{"tools":[…]}`, one member and no other, so that it
/// decodes as neither the word `"any"` nor a tags test. Each is a [`Key`]: the template and index that are one tool's on the daemon holding it.
/// It reaches those and no other, whatever their tags, judged at every
/// request; one that does not exist, or no longer does, is reached by
/// nothing. An empty list reaches nothing. See
/// [`Reach`](crate::shared::permission::Reach).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "StandaloneToolsListed")]
#[serde(deny_unknown_fields)]
pub struct Listed {
    /// The standalone tools reached.
    pub tools: Vec<Key>,
}
