//! The providers a grant lists.

use serde::{Deserialize, Serialize};

use crate::shared::Identity;

/// Exactly the providers listed, as a grant's `within` names them: the
/// object `{"identities":[…]}`, one member and no other, so that it
/// decodes as neither the word `"any"` nor a tags test. Each is a provider by its identity: the grant acts on the Stores of the providers listed.
/// It reaches those and no other, whatever their tags, judged at every
/// request; one that does not exist, or no longer does, is reached by
/// nothing. An empty list reaches nothing. See
/// [`Reach`](crate::shared::permission::Reach).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "ImagesListed")]
#[serde(deny_unknown_fields)]
pub struct Listed {
    /// The providers reached.
    pub identities: Vec<Identity>,
}
