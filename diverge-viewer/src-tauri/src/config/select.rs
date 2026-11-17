//! Which daemon, and which account of it.

use super::{Account, Config, Daemon, Error};

/// The daemon and the account a start is: what its arguments named,
/// else the defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selected {
    /// The daemon's name, which the window's title shows.
    pub daemon_name: String,
    /// The daemon.
    pub daemon: Daemon,
    /// The account.
    pub account: Account,
}

/// The daemon `daemon` names, else the default daemon, and the
/// account `account` names under it, else the daemon's default
/// account — as the CLI chooses for a `run`. A name the settings do
/// not have is [`Error::NoDaemon`] or [`Error::NoAccount`]; no
/// account named, and none the daemon's default, is
/// [`Error::NoDefaultAccount`].
pub fn select(config: &Config, daemon: Option<&str>, account: Option<&str>) -> Result<Selected, Error> {
    let daemon_name = daemon.unwrap_or(&config.default);
    let Some(daemon) = config.daemons.get(daemon_name) else {
        return Err(Error::NoDaemon(daemon_name.to_string()));
    };
    let Some(account_name) = account.or(daemon.default.as_deref()) else {
        return Err(Error::NoDefaultAccount(daemon_name.to_string()));
    };
    let Some(account) = daemon.accounts.get(account_name) else {
        return Err(Error::NoAccount { daemon: daemon_name.to_string(), account: account_name.to_string() });
    };
    Ok(Selected { daemon_name: daemon_name.to_string(), daemon: daemon.clone(), account: account.clone() })
}
