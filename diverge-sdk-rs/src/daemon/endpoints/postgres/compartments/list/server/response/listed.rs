//! One compartment, as a list reports it.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::daemon::endpoints::postgres::compartments::{Compartment, Kind};
use crate::shared::listing::Keyed;

/// One compartment as a list and a get report it: which it is, what
/// kind of container it is of, its tags, since when it is on record,
/// whether a delete would be allowed now, and how many connections
/// containers hold through it now.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "PostgresCompartmentListed")]
pub struct Listed {
    /// Which compartment, an agent in it by its template and index:
    /// see [`Compartment`].
    pub compartment: Compartment,
    /// What kind of container it is of: see [`Kind`]. An `image`
    /// compartment's is the `type` of the image's document.
    pub kind: Kind,
    /// Its tags, sorted bytewise: what
    /// [`tag`](crate::daemon::endpoints::postgres::compartments::tag)
    /// put on it and
    /// [`untag`](crate::daemon::endpoints::postgres::compartments::untag)
    /// has not taken off. Absent when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Since when it is on record: when the first agent, tool or
    /// dependency tool it is of was made. On the wire an RFC 3339
    /// timestamp in UTC.
    pub created: DateTime<Utc>,
    /// Whether a
    /// [`delete`](crate::daemon::endpoints::postgres::compartments::delete)
    /// would be allowed now, connections apart: see
    /// [`compartments`](crate::daemon::endpoints::postgres::compartments)
    /// for the rules.
    pub deletable: bool,
    /// How many connections containers hold through it now. While it
    /// is not `0` a delete answers `InUse`.
    pub connections: u64,
}

/// Keyed by the compartment itself.
impl Keyed for Listed {
    type Key = Compartment;

    fn key(&self) -> Compartment {
        self.compartment.clone()
    }
}
