//! What is done to tool templates that exist.

use serde::{Deserialize, Serialize};

/// The actions over tool templates that exist, which a grant reaches as
/// far as its `within` says. Snake case on the wire: `"get"`,
/// `"arguments"`, `"delete"`, `"list"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Over {
    /// Get one, as
    /// [`tools::templates::get`](crate::daemon::endpoints::standalone_tools::templates::get)
    /// does.
    Get,
    /// See one's arguments: have a
    /// [`get`](crate::daemon::endpoints::standalone_tools::templates::get) or a
    /// [`list`](crate::daemon::endpoints::standalone_tools::templates::list)
    /// whose request says `"arguments": true` answered with them. It
    /// is held beside `get` or `list` and never in place of either:
    /// alone it answers nothing.
    Arguments,
    /// Delete one, as
    /// [`tools::templates::delete`](crate::daemon::endpoints::standalone_tools::templates::delete)
    /// does.
    Delete,
    /// List them, as
    /// [`tools::templates::list`](crate::daemon::endpoints::standalone_tools::templates::list)
    /// does; the list sends what the grant reaches.
    List,
}
