//! Who listens.

use diverge_sdk::shared::Identity;

use crate::judge::{ToolRef, Who};
use crate::store::{AccountId, DependencyToolId, ToolId};

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
            Who::Tool(declared) => match declared.tool {
                ToolRef::Standalone(id) => Party::Standalone(id),
                ToolRef::Dependency(id) => Party::Dependency(id),
            },
        }
    }
}
