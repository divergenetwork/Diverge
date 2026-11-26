//! What is done to compartments.

use serde::{Deserialize, Serialize};

/// The actions over compartments, which a grant reaches as far as its
/// `within` says. Snake case on the wire: `"get"`, `"list"`,
/// `"delete"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "PostgresCompartmentsOver")]
#[serde(rename_all = "snake_case")]
pub enum Over {
    /// Get one, as
    /// [`compartments::get`](crate::daemon::endpoints::postgres::compartments::get)
    /// does.
    Get,
    /// List them, as
    /// [`compartments::list`](crate::daemon::endpoints::postgres::compartments::list)
    /// does; the list sends what the grant reaches.
    List,
    /// Delete one, with every table in it, as
    /// [`compartments::delete`](crate::daemon::endpoints::postgres::compartments::delete)
    /// does.
    Delete,
}
