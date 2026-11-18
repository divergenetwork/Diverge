//! The volumes a grant lists.

use serde::{Deserialize, Serialize};

use super::Key;

/// Exactly the volumes listed, as a grant's `within` names them: the
/// object `{"volumes":[…]}`, one member and no other, so that it
/// decodes as neither the word `"any"` nor a tags test. Each is a [`Key`]: the provider that holds the volume, and its name there.
/// It reaches those and no other, whatever their tags, judged at every
/// request; one that does not exist, or no longer does, is reached by
/// nothing. An empty list reaches nothing. See
/// [`Reach`](crate::shared::permission::Reach).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "VolumesListed")]
#[serde(deny_unknown_fields)]
pub struct Listed {
    /// The volumes reached.
    pub volumes: Vec<Key>,
}
