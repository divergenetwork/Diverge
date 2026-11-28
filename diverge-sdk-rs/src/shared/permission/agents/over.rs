//! What is done to agents that exist.

use serde::{Deserialize, Serialize};

/// The actions over agents that exist, which a grant reaches as far as
/// its `within` says. Snake case on the wire: `"get"`, `"delete"`,
/// `"edit"`, `"message"`, `"logs"`, `"list"`, `"download"`, `"upload"`,
/// `"transfer"`, `"filetree"`, `"exec"`, `"connect"`,
/// `"request"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Over {
    /// Get one, as
    /// [`agents::get`](crate::daemon::endpoints::agents::get) does.
    Get,
    /// Delete one, as
    /// [`agents::delete`](crate::daemon::endpoints::agents::delete)
    /// does.
    Delete,
    /// Change one, as
    /// [`agents::edit`](crate::daemon::endpoints::agents::edit) does.
    Edit,
    /// Send one a message, as
    /// [`agents::message`](crate::daemon::endpoints::agents::message)
    /// does.
    Message,
    /// Read one's log, as
    /// [`logs`](crate::daemon::endpoints::logs) does.
    Logs,
    /// List them, as
    /// [`agents::list`](crate::daemon::endpoints::agents::list) does;
    /// the list sends what the grant reaches.
    List,
    /// Send the client files out of one, as
    /// [`agents::download`](crate::daemon::endpoints::agents::download)
    /// does.
    Download,
    /// Put files into one, as
    /// [`agents::upload`](crate::daemon::endpoints::agents::upload)
    /// does, and as a transfer landing in one does.
    Upload,
    /// Copy files out of one, as
    /// [`agents::transfer`](crate::daemon::endpoints::agents::transfer)
    /// does; where they land is judged by its own grant.
    Transfer,
    /// Watch one's container whole, as
    /// [`agents::filetree`](crate::daemon::endpoints::agents::filetree)
    /// does.
    Filetree,
    /// Run a command in one's container — any program it holds, as
    /// the container's own user — as
    /// [`agents::exec`](crate::daemon::endpoints::agents::exec) does.
    Exec,
    /// Run MCP exchanges as one — list and call its tools, list and
    /// read its resources, as its own container would — as
    /// [`agents::connect`](crate::daemon::endpoints::agents::connect)
    /// does: the scope deploys its container and holds it up while it
    /// is open, and makes it neither active nor busy.
    Connect,
    /// Make a request of them, as
    /// [`requests::create`](crate::daemon::endpoints::requests::create)
    /// does: a request is made of an agent it addresses — by itself,
    /// by a tag, or as one of all — only where the grant reaches it.
    Request,
}
