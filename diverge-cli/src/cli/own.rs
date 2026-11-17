//! The daemons a program may dial, a setting at a time: the list,
//! what each of them is, and which is the default. The CLI keeps
//! such a list for itself and the viewer keeps one of its own, member
//! for member the same, so both are worked on here: `diverge cli
//! daemons …` and `diverge viewer daemons …`, `diverge cli default …`
//! and `diverge viewer default …`.

use std::path::Path;

use diverge_sdk::shared;
use indexmap::IndexMap;
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::Error;
use super::setting::{hold, line, read, write};
use crate::arguments::{Chosen, DaemonAccounts, Daemons, DaemonsCommand, Identity, Text};
use crate::config::{self, Account, Daemon};
use crate::document::{self, Which};
use crate::lifecycle::Program;

/// Settings that hold daemons by name and name one of them the
/// default: the CLI's own, and the viewer's.
pub trait Book: Serialize + DeserializeOwned {
    /// Which program's settings these are.
    const WHICH: Which;
    /// The program that must not be running on the root while one is
    /// written: the viewer, for the viewer's. The CLI's have none: the
    /// CLI reads its own at every command and holds no lock.
    const PROGRAM: Option<Program>;
    /// What a command of the list says when it was given an index.
    const LISTWISE: &'static str;
    /// What a command of one daemon says when it was given none.
    const ONE: &'static str;
    /// The command that makes another daemon the default.
    const DEFAULT_DAEMON: &'static str;
    /// The command that makes another account a daemon's default.
    const DEFAULT_ACCOUNT: &'static str;

    /// The default daemon's name, and the daemons.
    fn book(&mut self) -> (&mut String, &mut IndexMap<String, Daemon>);
}

impl Book for config::Config {
    const WHICH: Which = Which::Cli;
    const PROGRAM: Option<Program> = None;
    const LISTWISE: &'static str = "`get`, `add` and `delete` work on the list of daemons and take no daemon's index before them: `diverge cli daemons get`";
    const ONE: &'static str = "name the daemon by its index first, as `diverge cli daemons get` prints it: `diverge cli daemons 0 address get`";
    const DEFAULT_DAEMON: &'static str = "diverge cli default set <INDEX>";
    const DEFAULT_ACCOUNT: &'static str = "diverge cli daemons <INDEX> default set <INDEX>";

    fn book(&mut self) -> (&mut String, &mut IndexMap<String, Daemon>) {
        (&mut self.default, &mut self.daemons)
    }
}

impl Book for config::Viewer {
    const WHICH: Which = Which::Viewer;
    const PROGRAM: Option<Program> = Some(Program::Viewer);
    const LISTWISE: &'static str = "`get`, `add` and `delete` work on the list of daemons and take no daemon's index before them: `diverge viewer daemons get`";
    const ONE: &'static str = "name the daemon by its index first, as `diverge viewer daemons get` prints it: `diverge viewer daemons 0 address get`";
    const DEFAULT_DAEMON: &'static str = "diverge viewer default set <INDEX>";
    const DEFAULT_ACCOUNT: &'static str = "diverge viewer daemons <INDEX> default set <INDEX>";

    fn book(&mut self) -> (&mut String, &mut IndexMap<String, Daemon>) {
        (&mut self.default, &mut self.daemons)
    }
}

/// `diverge cli daemons …` and `diverge viewer daemons …`: the
/// daemons the program may dial, and each of them. The daemons are
/// kept by name, in an order, and each daemon's accounts likewise;
/// here both are lists in that order, each item with its name as a
/// member, so that they are worked on as every other list is. By
/// these rules:
///
/// - **The list**, with no index before the command. `get` prints
///   each daemon as one JSON line, its `index` and its `name`, and
///   nothing more of it. `add` puts a daemon after the last, with the
///   address given and no account, no default account and no identity
///   of its own; a name that is a daemon's already is refused,
///   [`Error::Listed`]. `delete` takes the index and removes the
///   daemon with its accounts; the default daemon is refused,
///   [`Error::Default`], since a `default` that names no daemon dials
///   none, and which to dial instead is not the delete's to choose.
/// - **One daemon**, with its index before the command. What a daemon
///   is, is read a setting at a time, and there is no reading of a
///   daemon whole:
///   - `address get` prints the string, alone on its line; `address
///     set` writes the one given.
///   - `identity get` prints `{"issuer":"…","subject":"…"}`, or
///     `null` where none is stated; `identity set` takes both and
///     writes the identity whole, and none makes it absent.
///   - `accounts get` prints each account as one JSON line, its
///     `index`, `name`, `issuer` and `subject`. `accounts add` puts
///     one after the last, and refuses a name that is an account's of
///     the daemon already. `accounts delete` takes the index, and
///     refuses the daemon's default account, as the daemons' delete
///     refuses the default daemon.
///   - `default get` prints the daemon's default account,
///     `{"index":…,"name":"…"}`, or `null` where the daemon has none
///     or names one it does not have; `default set` takes an
///     account's index and writes that account's name.
/// - A command of the list given an index, and a command of one
///   daemon given none, is refused, [`Error::Usage`]; an index the
///   list does not reach is refused, [`Error::NoItem`].
/// - A write is by [`set`]'s rules: the viewer's is refused while
///   the viewer runs on the root; the CLI's takes no lock. Nothing is
///   printed by a write.
pub async fn daemons<B: Book>(root: &Path, action: Daemons) -> Result<(), Error> {
    let Daemons { index, command } = action;
    let listwise = matches!(command, DaemonsCommand::Get | DaemonsCommand::Add { .. } | DaemonsCommand::Delete { .. });
    match (listwise, index) {
        (true, Some(_)) => return Err(Error::Usage(B::LISTWISE)),
        (false, None) => return Err(Error::Usage(B::ONE)),
        _ => {}
    }
    let at = index.unwrap_or_default();
    match command {
        DaemonsCommand::Get => {
            let mut config = read::<B>(root, B::WHICH).await?;
            let mut text = Vec::new();
            for (index, name) in config.book().1.keys().enumerate() {
                let line = serde_json::json!({"index": index, "name": name});
                serde_json::to_writer(&mut text, &line).map_err(document::Error::Json)?;
                text.push(b'\n');
            }
            write(text).await
        }
        DaemonsCommand::Add { name, address } => {
            set::<B>(root, |_, daemons| {
                if let Some(index) = daemons.get_index_of(&name) {
                    return Err(Error::Listed { list: "daemons", item: format!("a daemon named `{name}`"), index });
                }
                daemons.insert(name, Daemon { address, default: None, identity: None, accounts: IndexMap::new() });
                Ok(())
            })
            .await
        }
        DaemonsCommand::Delete { index } => {
            set::<B>(root, |default, daemons| {
                let Some((name, _)) = daemons.get_index(index) else {
                    return Err(Error::NoItem { list: "daemons", index, count: daemons.len() });
                };
                if name == default {
                    return Err(Error::Default { what: "daemon", name: name.clone(), index, instead: B::DEFAULT_DAEMON });
                }
                // Shifted, not swapped: every daemon after it moves
                // one index down, and the order is otherwise kept.
                daemons.shift_remove_index(index);
                Ok(())
            })
            .await
        }
        DaemonsCommand::Address(Text::Get) => {
            let mut config = read::<B>(root, B::WHICH).await?;
            write(format!("{}\n", one(config.book().1, at)?.address).into_bytes()).await
        }
        DaemonsCommand::Address(Text::Set { value }) => {
            set::<B>(root, |_, daemons| {
                one(daemons, at)?.address = value;
                Ok(())
            })
            .await
        }
        DaemonsCommand::Identity(Identity::Get) => {
            let mut config = read::<B>(root, B::WHICH).await?;
            line(&one(config.book().1, at)?.identity).await
        }
        DaemonsCommand::Identity(Identity::Set { issuer, subject }) => {
            set::<B>(root, |_, daemons| {
                one(daemons, at)?.identity = Some(shared::Identity { issuer, subject });
                Ok(())
            })
            .await
        }
        DaemonsCommand::Default(Chosen::Get) => {
            let mut config = read::<B>(root, B::WHICH).await?;
            let daemon = one(config.book().1, at)?;
            let chosen = daemon.default.as_ref().and_then(|name| {
                daemon.accounts.get_index_of(name).map(|index| serde_json::json!({"index": index, "name": name}))
            });
            line(&chosen).await
        }
        DaemonsCommand::Default(Chosen::Set { index }) => {
            set::<B>(root, |_, daemons| {
                let daemon = one(daemons, at)?;
                let Some((name, _)) = daemon.accounts.get_index(index) else {
                    return Err(Error::NoItem { list: "accounts", index, count: daemon.accounts.len() });
                };
                daemon.default = Some(name.clone());
                Ok(())
            })
            .await
        }
        DaemonsCommand::Accounts(DaemonAccounts::Get) => {
            let mut config = read::<B>(root, B::WHICH).await?;
            let mut text = Vec::new();
            for (index, (name, account)) in one(config.book().1, at)?.accounts.iter().enumerate() {
                let line = serde_json::json!({"index": index, "name": name, "issuer": account.issuer, "subject": account.subject});
                serde_json::to_writer(&mut text, &line).map_err(document::Error::Json)?;
                text.push(b'\n');
            }
            write(text).await
        }
        DaemonsCommand::Accounts(DaemonAccounts::Add { name, issuer, subject }) => {
            set::<B>(root, |_, daemons| {
                let daemon = one(daemons, at)?;
                if let Some(index) = daemon.accounts.get_index_of(&name) {
                    return Err(Error::Listed { list: "accounts", item: format!("an account named `{name}`"), index });
                }
                daemon.accounts.insert(name, Account { issuer, subject });
                Ok(())
            })
            .await
        }
        DaemonsCommand::Accounts(DaemonAccounts::Delete { index }) => {
            set::<B>(root, |_, daemons| {
                let daemon = one(daemons, at)?;
                let Some((name, _)) = daemon.accounts.get_index(index) else {
                    return Err(Error::NoItem { list: "accounts", index, count: daemon.accounts.len() });
                };
                if daemon.default.as_ref() == Some(name) {
                    return Err(Error::Default { what: "account", name: name.clone(), index, instead: B::DEFAULT_ACCOUNT });
                }
                daemon.accounts.shift_remove_index(index);
                Ok(())
            })
            .await
        }
    }
}

/// The daemon at `index` in the list's order, or [`Error::NoItem`]
/// where the list does not reach it.
fn one(daemons: &mut IndexMap<String, Daemon>, index: usize) -> Result<&mut Daemon, Error> {
    let count = daemons.len();
    match daemons.get_index_mut(index) {
        Some((_, daemon)) => Ok(daemon),
        None => Err(Error::NoItem { list: "daemons", index, count }),
    }
}

/// `diverge cli default get|set` and `diverge viewer default
/// get|set`: which daemon is dialled when none is named. It is kept
/// by name; here it is said by its index among the daemons, as
/// `daemons get` prints them:
///
/// - `get` prints `{"index":…,"name":"…"}`, or `null` where the
///   `default` names no daemon there is.
/// - `set` takes an index and writes that daemon's name as the
///   default. An index the list does not reach is refused,
///   [`Error::NoItem`].
pub async fn default<B: Book>(root: &Path, action: Chosen) -> Result<(), Error> {
    match action {
        Chosen::Get => {
            let mut config = read::<B>(root, B::WHICH).await?;
            let (default, daemons) = config.book();
            let found = daemons.get_index_of(default.as_str());
            line(&found.map(|index| serde_json::json!({"index": index, "name": default}))).await
        }
        Chosen::Set { index } => {
            set::<B>(root, |default, daemons| {
                let Some((name, _)) = daemons.get_index(index) else {
                    return Err(Error::NoItem { list: "daemons", index, count: daemons.len() });
                };
                *default = name.clone();
                Ok(())
            })
            .await
        }
    }
}

/// The settings read as their type, changed, and written whole. A
/// change that refuses writes nothing. Where the settings have a
/// program — the viewer's — its lock on the root is taken first and
/// kept until the write is done, and a lock that is held is the
/// program running, [`Error::Stop`]: see [`hold`].
async fn set<B: Book>(
    root: &Path,
    change: impl FnOnce(&mut String, &mut IndexMap<String, Daemon>) -> Result<(), Error>,
) -> Result<(), Error> {
    let held = match B::PROGRAM {
        Some(program) => Some(hold(root, program).await?),
        None => None,
    };
    let mut config = read::<B>(root, B::WHICH).await?;
    let (default, daemons) = config.book();
    change(default, daemons)?;
    let yaml = serde_yaml_ng::to_string(&config).map_err(document::Error::Serialize)?;
    document::write(root, B::WHICH, &yaml).await?;
    drop(held);
    Ok(())
}
