//! Whose book a request is kept in.

use crate::containers::ToolKey;
use crate::judge::{Container, Who};
use crate::store::{AccountId, AgentId};
use crate::store::hash;

/// An administrator, by its identity's
/// [`identity_key`](crate::store::hash::identity_key): it has no row,
/// and its book is named by the key in hex.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AdminKey(pub [u8; 32]);

/// Who put a request to the daemon, as its book is named.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Commander {
    /// An account, by its row.
    Account(AccountId),
    /// An administrator, by its identity's key.
    Admin(AdminKey),
    /// A tool's container: its commands go into the tool's own log.
    Tool(ToolKey),
    /// An agent's container: its commands go into the agent's own log.
    Agent(AgentId),
}

impl Commander {
    /// The requester `who` is. An administrator whose identity cannot
    /// be hashed has no book.
    pub fn of(who: &Who) -> Option<Commander> {
        match who {
            Who::Account(id) => Some(Commander::Account(*id)),
            Who::Admin(identity) => hash::identity_key(identity).ok().map(|key| Commander::Admin(AdminKey(key))),
            Who::Container(declared) => Some(match declared.container {
                Container::Tool(tool) => Commander::Tool(tool.key()),
                Container::Agent(agent) => Commander::Agent(agent),
            }),
        }
    }
}
