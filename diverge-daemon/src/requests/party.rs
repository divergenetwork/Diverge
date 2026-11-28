//! Who listens.

use diverge_sdk::shared::Identity;

use crate::judge::{Container, ToolRef, Who};
use crate::store::{AccountId, AgentId, DependencyToolId, ToolId};

/// A party that may listen for requests, as the daemon tells one from
/// another: what the one listen a party may have open is counted by,
/// and what a request's deliveries are counted by. It is the caller,
/// whatever connection it is on.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Party {
    /// An account, by its row.
    Account(AccountId),
    /// An administrator the configuration names, which has no row.
    Admin(Identity),
    /// An agent, by its row.
    Agent(AgentId),
    /// A standalone tool, by its row.
    Standalone(ToolId),
    /// A dependency tool, by its row.
    Dependency(DependencyToolId),
}

impl Party {
    /// The party a caller is.
    pub fn of(who: &Who) -> Party {
        match who {
            Who::Account(id) => Party::Account(*id),
            Who::Admin(identity) => Party::Admin(identity.clone()),
            Who::Container(declared) => match declared.container {
                Container::Agent(id) => Party::Agent(id),
                Container::Tool(ToolRef::Standalone(id)) => Party::Standalone(id),
                Container::Tool(ToolRef::Dependency(id)) => Party::Dependency(id),
            },
        }
    }
}
