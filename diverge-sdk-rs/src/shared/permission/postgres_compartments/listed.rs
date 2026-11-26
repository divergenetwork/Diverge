//! The compartments a grant lists.

use serde::{Deserialize, Serialize};

use crate::daemon::endpoints::postgres::compartments::Compartment;

/// Exactly the compartments listed, as a grant's `within` names them:
/// the object `{"compartments":[…]}`, one member and no other, so that
/// it decodes as neither the word `"any"` nor a tags test. Each is a
/// [`Compartment`], an agent in it by its template and index: one
/// naming an agent by name reaches nothing. It reaches those and no
/// other, whatever their tags, judged at every request. An empty list
/// reaches nothing. See [`Reach`](crate::shared::permission::Reach).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "PostgresCompartmentsListed")]
#[serde(deny_unknown_fields)]
pub struct Listed {
    /// The compartments reached.
    pub compartments: Vec<Compartment>,
}
