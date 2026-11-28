//! What is done to dependency templates that exist, and to the tools
//! running from them.

use serde::{Deserialize, Serialize};

/// The actions over dependency tools that exist, which a grant
/// reaches as far as its `within` says: each tool is judged on its
/// own, by the tools list filter.
/// Snake case on the wire: `"get"`, `"list"`, `"arguments"`,
/// `"download"`, `"upload"`, `"transfer"`, `"filetree"`, `"exec"`, `"frontend"`, `"api"`, `"logs"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Over {
    /// Get one, as
    /// [`dependencies::get`](crate::daemon::endpoints::dependency_tools::get)
    /// does.
    Get,
    /// List them, as
    /// [`dependencies::list`](crate::daemon::endpoints::dependency_tools::list)
    /// does; the list sends what the grant reaches.
    List,
    /// See the arguments of one's template: have a
    /// [`get`](crate::daemon::endpoints::dependency_tools::get) or a
    /// [`list`](crate::daemon::endpoints::dependency_tools::list)
    /// whose request says `"arguments": true` answered with them. It
    /// is held beside `get` or `list` and never in place of either:
    /// alone it answers nothing.
    Arguments,
    /// Send the client files out of one, running, as
    /// [`dependencies::download`](crate::daemon::endpoints::dependency_tools::download)
    /// does.
    Download,
    /// Put files into one, running, as
    /// [`dependencies::upload`](crate::daemon::endpoints::dependency_tools::upload)
    /// does, and as a transfer landing in one does.
    Upload,
    /// Copy files out of one, running, as
    /// [`dependencies::transfer`](crate::daemon::endpoints::dependency_tools::transfer)
    /// does; where they land is judged by its own grant.
    Transfer,
    /// Watch one's container whole, as
    /// [`dependencies::filetree`](crate::daemon::endpoints::dependency_tools::filetree)
    /// does.
    Filetree,
    /// Run a command in one's container — any program it holds, as
    /// the container's own user — as
    /// [`dependency_tools::exec`](crate::daemon::endpoints::dependency_tools::exec) does.
    Exec,
    /// Send HTTP requests to the frontend its program serves, as
    /// [`dependency_tools::frontend`](crate::daemon::endpoints::dependency_tools::frontend) does.
    Frontend,
    /// Send HTTP requests to the API its program serves, as
    /// [`dependency_tools::api`](crate::daemon::endpoints::dependency_tools::api) does.
    Api,
    /// Read one's log, running or not, as
    /// [`logs`](crate::daemon::endpoints::logs)
    /// does.
    Logs,
    /// Make a request of them, as
    /// [`requests::create`](crate::daemon::endpoints::requests::create)
    /// does: a request is made of a tool it addresses — by itself, by
    /// a tag, or as one of all — only where the grant reaches it.
    Request,
}
