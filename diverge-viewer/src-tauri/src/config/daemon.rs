//! One daemon the app may dial, and one account of it.

use diverge_sdk::shared::Identity;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// A daemon the app may dial: where, and as whom. The same members
/// the CLI keeps for a daemon of its own.
///
/// The settings hold no token: the app reads the account's identity
/// at start and keeps it fresh, and it never reaches the page, which
/// speaks to the daemon only through the app's own Rust.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Daemon {
    /// Where it is dialled: a WebSocket URL, or a bare `host:port`
    /// the app puts `ws://` before and `/` after.
    pub address: String,
    /// The account dialled as when the start names none, by its name
    /// in `accounts`. Absent, a start must name one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    /// The daemon's own identity, `{issuer, subject}`: who must answer
    /// at the address. Stated, the handshake fails unless that
    /// identity answers and proves the key its token names. Absent,
    /// whichever identity answers with a token that verifies is taken.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<Identity>,
    /// The accounts the app may dial it as, by name, in the order
    /// they were added.
    #[serde(default)]
    pub accounts: IndexMap<String, Account>,
}

/// An account the app may dial a daemon as: one of the root's
/// [`identities`](diverge_sdk::identities), signed in by
/// `diverge identities login`, whose `(issuer, subject)` is the
/// daemon's account. What the app shows is what that account may
/// see.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Account {
    /// The identity's issuer: the one the app presents its token
    /// from.
    pub issuer: String,
    /// The identity's subject at that issuer.
    pub subject: String,
}
