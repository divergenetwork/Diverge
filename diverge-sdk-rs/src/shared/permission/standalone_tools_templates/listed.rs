//! The standalone tool templates a grant lists.

use serde::{Deserialize, Serialize};

/// Exactly the standalone tool templates listed, as a grant's `within` names them: the
/// object `{"ids":[…]}`, one member and no other, so that it
/// decodes as neither the word `"any"` nor a tags test. Each is a template by its id.
/// It reaches those and no other, whatever their tags, judged at every
/// request; one that does not exist, or no longer does, is reached by
/// nothing. An empty list reaches nothing. See
/// [`Reach`](crate::shared::permission::Reach).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "StandaloneToolsTemplatesListed")]
#[serde(deny_unknown_fields)]
pub struct Listed {
    /// The standalone tool templates reached.
    pub ids: Vec<String>,
}
