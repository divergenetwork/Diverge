//! Who is calling, on a request.

use serde::{Deserialize, Serialize};

use super::{Agent, Tool};
use crate::shared::Identity;

/// The caller, as the daemon attests it on the `_meta` of the four
/// requests a tool receives — `list_tools`, `list_resources`,
/// `call_tool`, `read_resource` — and on the `diverge.network` header
/// of every frontend request and every API request (see
/// [`header`](crate::shared::mcp::header)). Tagged `kind`, three kinds,
/// one for each kind of requester a daemon has. The rules:
///
/// 1. **An agent.** An [`Agent`] of the daemon that runs the tool:
///    `kind` `agent`, with its image and its key. It is the same
///    caller whichever way the agent comes — an MCP call its
///    container sent, or a connect, a frontend request or an API
///    request it put to the daemon over its own daemon connection —
///    so a tool knows one agent by one caller. It carries no scope.
/// 2. **A client.** An account of that daemon's reaching the tool over
///    a connect, a frontend request or an API request: `kind`
///    `client`, the identity the daemon established at the handshake
///    and the scope's id, and nothing more — not the agent the client
///    may say is calling, not its image.
/// 3. **A tool.** A tool container of that daemon's reaching the tool
///    over its own daemon connection, by a connect, a frontend request
///    or an API request: `kind` `tool`, the tool as the daemon attests
///    one, and the scope's id.
/// 4. **Nothing a caller says of itself is trusted.** Every member is
///    the daemon's own word, set in place of whatever the caller sent.
/// 5. **A scope is one scope's.** A client's and a tool's `scope` is
///    minted for the one connect, frontend or API scope the call came
///    on, so two scopes of one caller are two callers to a tool, and
///    two MCP sessions. An agent has none, and one session with each
///    tool, whichever way it comes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Caller {
    /// An agent of this daemon's, calling from its container:
    /// `{"kind":"agent","image":{…},"template":…,"index":…,"name":…}`.
    Agent(Agent),
    /// A client of this daemon's, calling over a connect, a frontend
    /// request or an API request:
    /// `{"kind":"client","identity":{"issuer":…,"subject":…},"scope":…}`.
    Client {
        /// The account the connection is: the identity its token
        /// established — as every `Creator::Client` names the same
        /// client.
        identity: Identity,
        /// The scope the call came on: a random version 4 UUID,
        /// lowercase and hyphenated, which the daemon minted for that
        /// one connect, frontend or API scope. Every call on one scope
        /// carries the same; nothing is kept to make two scopes'
        /// differ but the randomness.
        scope: String,
    },
    /// A tool container of this daemon's, calling over its own daemon
    /// connection by a connect, a frontend request or an API request:
    /// `{"kind":"tool","tool":{"kind":"created",…},"scope":…}`.
    Tool {
        /// The tool calling, as the daemon attests a tool: its kind —
        /// `created` or `dependency`; a connected tool runs on another
        /// daemon and never calls here — its image, its template, and
        /// its index or its owner. See [`Tool`].
        tool: Tool,
        /// The scope the call came on, as a client's is: a random
        /// version 4 UUID the daemon minted for that one connect,
        /// frontend or API scope.
        scope: String,
    },
}
