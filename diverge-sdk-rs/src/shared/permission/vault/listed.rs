//! The vault entries a grant lists.

use serde::{Deserialize, Serialize};

/// Exactly the vault entries listed, as a grant's `within` names them: the
/// object `{"keys":[…]}`, one member and no other, so that it
/// decodes as neither the word `"any"` nor a tags test. Each is a vault entry by its key.
/// It reaches those and no other, whatever their tags, judged at every
/// request; one that does not exist, or no longer does, is reached by
/// nothing. An empty list reaches nothing. See
/// [`Reach`](crate::shared::permission::Reach).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "VaultListed")]
#[serde(deny_unknown_fields)]
pub struct Listed {
    /// The vault entries reached.
    pub keys: Vec<String>,
}
