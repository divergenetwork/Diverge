//! One compartment, as the store holds it.

use chrono::{DateTime, Utc};
use diverge_sdk::daemon::endpoints::postgres::compartments::list::server::response::Listed;
use diverge_sdk::daemon::endpoints::postgres::compartments::{Compartment, Kind};

/// A compartment row.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// Its role, and its schema: what names the row.
    pub role: String,
    /// The compartment, an agent in it by its template and index.
    pub compartment: Compartment,
    /// What kind of container it is of.
    pub kind: Kind,
    /// Its tags, sorted bytewise.
    pub tags: Vec<String>,
    /// Since when it is on record.
    pub created: DateTime<Utc>,
}

impl Entry {
    /// The compartment as a list reports it, with whether a delete
    /// would be allowed now and how many connections are held through
    /// it, each found by the caller.
    pub fn report(&self, deletable: bool, connections: u64) -> Listed {
        Listed {
            compartment: self.compartment.clone(),
            kind: self.kind,
            tags: self.tags.clone(),
            created: self.created,
            deletable,
            connections,
        }
    }
}

/// The kind's word, as the `kind` column holds it: the snake case of
/// the wire.
pub fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Agent => "agent",
        Kind::StandaloneTool => "standalone_tool",
        Kind::DependencyTool => "dependency_tool",
    }
}

/// The kind a column's word names, or `None` for a word that is none
/// of the three.
pub fn kind_of(name: &str) -> Option<Kind> {
    match name {
        "agent" => Some(Kind::Agent),
        "standalone_tool" => Some(Kind::StandaloneTool),
        "dependency_tool" => Some(Kind::DependencyTool),
        _ => None,
    }
}
