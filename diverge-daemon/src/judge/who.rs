//! Who a connection is served for.

use diverge_sdk::shared::Identity;

use super::Declared;
use crate::store::AccountId;

/// Who is asking, fixed at the handshake: an account, by its id and
/// nothing else — not its identity and not its grants, since both may
/// change while the connection lives, a rename, a role edited, and
/// every request reads them fresh as a [`Standing`](super::Standing);
/// or a container — an agent's, or a tool's, standalone or dependency
/// — by what its program declared at its registration, which nothing
/// changes for the run, and which container it is, whose attachments
/// attaching and detaching change and every request reads fresh; or
/// an administrator, by its identity, which the configuration fixes
/// for the daemon's life.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Who {
    /// A client's account.
    Account(AccountId),
    /// A container, an agent's or a tool's, under the grants its
    /// program declared.
    Container(Declared),
    /// An administrator the configuration's `admins` names, by its
    /// identity: no account, and every grant there is.
    Admin(Identity),
}

impl Who {
    /// The account, when this is one.
    pub fn account(&self) -> Option<AccountId> {
        match self {
            Who::Account(id) => Some(*id),
            Who::Container(_) | Who::Admin(_) => None,
        }
    }
}
