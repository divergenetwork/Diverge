//! A login at Diverge's own issuer, which approves itself.
//!
//! An identity is signed in at any OpenID Connect issuer by a device
//! code, which a person approves there: the command prints the code
//! and leaves its wait behind — see [`detach`](crate::detach).
//! `identity.diverge.network` is Diverge's own, and a login there
//! needs no person: its subjects each name the account that owns
//! them, and where that account is signed in on this host, this
//! command approves the code itself. The rules:
//!
//! 1. **Only that issuer.** The login's issuer is
//!    [`ISSUER`](identity_diverge_network::ISSUER), exactly: the
//!    string `identity.diverge.network` signs its tokens with. Any
//!    other — that one with a trailing slash, another deployment's —
//!    is another issuer, and is signed in as any is.
//! 2. **The subject names its owner.** It is `<username>.<name>`,
//!    split at its first period — see
//!    [`parts`](identity_diverge_network::parts). One that is not is
//!    refused, before anything is asked of anybody.
//! 3. **The owner is signed in here.** The account whose username it
//!    is, is one of this host's — see
//!    [`Accounts::owner`](identity_diverge_network::Accounts::owner),
//!    which asks `identity.diverge.network` each account's username.
//!    Where none's is, it is [`Error::NotSignedIn`], which says how to
//!    sign one in and how to set its username.
//! 4. **The code is approved here.** Once the login has its user
//!    code, [`Ours::approve`] approves it as the subject's name, as
//!    the account: the account's token had under its lock, renewed
//!    where it has expired. The code is one this command began a
//!    moment before, and no other is ever approved. An account that
//!    has set no username can approve none —
//!    [`Error::NoUsername`].
//! 5. **Waited for.** Nothing is left behind: with nobody to wait on,
//!    the login ends in the time of a poll, and the command ends with
//!    it, its outcome printed.
//!
//! The login's two commands — on this host, and on a daemon's — each
//! ask [`ours`] first, and take this way where it answers one.

use std::fmt;
use std::path::Path;

use diverge_sdk::identity_diverge_network::{self, Account, Accounts};

/// A login at `identity.diverge.network` that this command can
/// approve: the account that owns its subject, signed in here.
pub struct Ours {
    accounts: Accounts,
    account: Account,
    name: String,
}

/// Why a login at `identity.diverge.network` could not be made here.
#[derive(Debug)]
pub enum Error {
    /// The subject is no subject of its, the accounts could not be
    /// opened, or its approval was refused.
    Accounts(identity_diverge_network::Error),
    /// No account signed in on this host has the subject's username:
    /// the username.
    NotSignedIn(String),
    /// The account has set no username, and so has no subject.
    NoUsername(Account),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Accounts(error) => write!(f, "{error}"),
            Error::NotSignedIn(username) => write!(
                f,
                "the subject is `{username}`'s, and no account signed in on this host has that username. Sign the account in with `diverge identity.diverge.network accounts login <ISSUER>`, enter the code it prints at the page it opens, find it with `diverge identity.diverge.network accounts list`, read its username with `diverge identity.diverge.network accounts username get <ISSUER> <ID>` — one that has none sets it once with `accounts username set <ISSUER> <ID> <USERNAME>` — and run this again"
            ),
            Error::NoUsername(account) => write!(
                f,
                "the account `{account}` has set no username, and a subject begins with one. Set it once, and for good, with `diverge identity.diverge.network accounts username set {} {} <USERNAME>`, and run this again",
                account.issuer, account.id
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Accounts(error) => Some(error),
            Error::NotSignedIn(_) | Error::NoUsername(_) => None,
        }
    }
}

impl From<identity_diverge_network::Error> for Error {
    fn from(error: identity_diverge_network::Error) -> Self {
        Error::Accounts(error)
    }
}

/// The login of `subject` at `issuer`, where it is one this command
/// approves itself — see the [rules](self), rules 1 to 3: `None` for
/// any issuer but `identity.diverge.network`'s own; the error for a
/// subject that is none of its, and for an owner not signed in here.
pub async fn ours(root: &Path, issuer: &str, subject: &str) -> Result<Option<Ours>, Error> {
    // Rule 1.
    if issuer != identity_diverge_network::ISSUER {
        return Ok(None);
    }
    // Rule 2.
    let (username, name) = identity_diverge_network::parts(subject)?;
    // Rule 3.
    let accounts = Accounts::open(root).await?;
    let Some(account) = accounts.owner(&username).await? else {
        return Err(Error::NotSignedIn(username));
    };
    Ok(Some(Ours { accounts, account, name }))
}

impl Ours {
    /// Approve `user_code`, the login's own, as the subject — see the
    /// [rules](self), rule 4.
    pub async fn approve(&self, user_code: &str) -> Result<(), Error> {
        match self.accounts.approve(&self.account, user_code, &self.name).await {
            Ok(_) => Ok(()),
            Err(identity_diverge_network::Error::Service { code, .. }) if code == "username_required" => Err(Error::NoUsername(self.account.clone())),
            Err(error) => Err(error.into()),
        }
    }
}
