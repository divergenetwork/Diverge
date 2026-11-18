//! The identities a grant lists.

use serde::{Deserialize, Serialize};

use crate::shared::Identity;

/// Exactly the identities listed, as a grant's `within` names them: the
/// object `{"identities":[…]}`, one member and no other, so that it
/// decodes as neither the word `"any"` nor a tags test. Each is one of the host's identities, by its issuer and its subject.
/// It reaches those and no other, whatever their tags, judged at every
/// request; one that does not exist, or no longer does, is reached by
/// nothing. An empty list reaches nothing. See
/// [`Reach`](crate::shared::permission::Reach).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "IdentitiesListed")]
#[serde(deny_unknown_fields)]
pub struct Listed {
    /// The identities reached.
    pub identities: Vec<Identity>,
}
