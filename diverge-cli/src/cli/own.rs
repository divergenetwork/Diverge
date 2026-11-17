//! The CLI's own document, a setting at a time: its daemons, what
//! each of them is, and which is the default. The only way the CLI
//! works on `cli.yaml`: there is no `cli config`.

use std::path::Path;

use diverge_sdk::shared;
use indexmap::IndexMap;

use super::Error;
use super::setting::{line, read, write};
use crate::arguments::{Chosen, DaemonAccounts, Daemons, DaemonsCommand, Identity, Text};
use crate::config::{Account, Config, Daemon};
use crate::document::{self, Which};

/// `diverge cli daemons …`: the daemons the CLI may dial, and each of
/// them. The document keeps the daemons by name, in an order, and
/// each daemon's accounts likewise; here both are lists in that
/// order, each item with its name as a member, so that they are
/// worked on as every other list is. By these rules:
///
/// - **The list**, with no index before the command. `get` prints
///   each daemon as one JSON line, its `index` and its `name`, and
///   nothing more of it. `add` puts a daemon after the last, with the
///   address given and no account, no default account and no identity
///   of its own; a name that is a daemon's already is refused,
///   [`Error::Listed`]. `delete` takes the index and removes the
///   daemon with its accounts; the default daemon is refused,
///   [`Error::Default`], since a document whose `default` names no
///   daemon dials none, and which to dial instead is not the delete's
///   to choose.
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
/// - No lock is taken: the CLI holds none on the root, and reads the
///   document at every command. Nothing is printed by a write.
pub async fn daemons(root: &Path, action: Daemons) -> Result<(), Error> {
    let Daemons { index, command } = action;
    let listwise = matches!(command, DaemonsCommand::Get | DaemonsCommand::Add { .. } | DaemonsCommand::Delete { .. });
    match (listwise, index) {
        (true, Some(_)) => {
            return Err(Error::Usage("`get`, `add` and `delete` work on the list of daemons and take no daemon's index before them: `diverge cli daemons get`"));
        }
        (false, None) => {
            return Err(Error::Usage("name the daemon by its index first, as `diverge cli daemons get` prints it: `diverge cli daemons 0 address get`"));
        }
        _ => {}
    }
    let at = index.unwrap_or_default();
    match command {
        DaemonsCommand::Get => {
            let config = read::<Config>(root, Which::Cli).await?;
            let mut text = Vec::new();
            for (index, name) in config.daemons.keys().enumerate() {
                let line = serde_json::json!({"index": index, "name": name});
                serde_json::to_writer(&mut text, &line).map_err(document::Error::Json)?;
                text.push(b'\n');
            }
            write(text).await
        }
        DaemonsCommand::Add { name, address } => {
            set(root, |config| {
                if let Some(index) = config.daemons.get_index_of(&name) {
                    return Err(Error::Listed { list: "daemons", item: format!("a daemon named `{name}`"), index });
                }
                config.daemons.insert(name, Daemon { address, default: None, identity: None, accounts: IndexMap::new() });
                Ok(())
            })
            .await
        }
        DaemonsCommand::Delete { index } => {
            set(root, |config| {
                let Some((name, _)) = config.daemons.get_index(index) else {
                    return Err(Error::NoItem { list: "daemons", index, count: config.daemons.len() });
                };
                if *name == config.default {
                    return Err(Error::Default {
                        what: "daemon",
                        name: name.clone(),
                        index,
                        instead: "diverge cli default set <INDEX>",
                    });
                }
                // Shifted, not swapped: every daemon after it moves
                // one index down, and the order is otherwise kept.
                config.daemons.shift_remove_index(index);
                Ok(())
            })
            .await
        }
        DaemonsCommand::Address(Text::Get) => {
            let config = read::<Config>(root, Which::Cli).await?;
            write(format!("{}\n", one(&config, at)?.address).into_bytes()).await
        }
        DaemonsCommand::Address(Text::Set { value }) => {
            set(root, |config| {
                one_mut(config, at)?.address = value;
                Ok(())
            })
            .await
        }
        DaemonsCommand::Identity(Identity::Get) => {
            let config = read::<Config>(root, Which::Cli).await?;
            line(&one(&config, at)?.identity).await
        }
        DaemonsCommand::Identity(Identity::Set { issuer, subject }) => {
            set(root, |config| {
                one_mut(config, at)?.identity = Some(shared::Identity { issuer, subject });
                Ok(())
            })
            .await
        }
        DaemonsCommand::Default(Chosen::Get) => {
            let config = read::<Config>(root, Which::Cli).await?;
            let daemon = one(&config, at)?;
            let chosen = daemon.default.as_ref().and_then(|name| {
                daemon.accounts.get_index_of(name).map(|index| serde_json::json!({"index": index, "name": name}))
            });
            line(&chosen).await
        }
        DaemonsCommand::Default(Chosen::Set { index }) => {
            set(root, |config| {
                let daemon = one_mut(config, at)?;
                let Some((name, _)) = daemon.accounts.get_index(index) else {
                    return Err(Error::NoItem { list: "accounts", index, count: daemon.accounts.len() });
                };
                daemon.default = Some(name.clone());
                Ok(())
            })
            .await
        }
        DaemonsCommand::Accounts(DaemonAccounts::Get) => {
            let config = read::<Config>(root, Which::Cli).await?;
            let mut text = Vec::new();
            for (index, (name, account)) in one(&config, at)?.accounts.iter().enumerate() {
                let line = serde_json::json!({"index": index, "name": name, "issuer": account.issuer, "subject": account.subject});
                serde_json::to_writer(&mut text, &line).map_err(document::Error::Json)?;
                text.push(b'\n');
            }
            write(text).await
        }
        DaemonsCommand::Accounts(DaemonAccounts::Add { name, issuer, subject }) => {
            set(root, |config| {
                let daemon = one_mut(config, at)?;
                if let Some(index) = daemon.accounts.get_index_of(&name) {
                    return Err(Error::Listed { list: "accounts", item: format!("an account named `{name}`"), index });
                }
                daemon.accounts.insert(name, Account { issuer, subject });
                Ok(())
            })
            .await
        }
        DaemonsCommand::Accounts(DaemonAccounts::Delete { index }) => {
            set(root, |config| {
                let daemon = one_mut(config, at)?;
                let Some((name, _)) = daemon.accounts.get_index(index) else {
                    return Err(Error::NoItem { list: "accounts", index, count: daemon.accounts.len() });
                };
                if daemon.default.as_ref() == Some(name) {
                    return Err(Error::Default {
                        what: "account",
                        name: name.clone(),
                        index,
                        instead: "diverge cli daemons <INDEX> default set <INDEX>",
                    });
                }
                daemon.accounts.shift_remove_index(index);
                Ok(())
            })
            .await
        }
    }
}

/// The daemon at `index` in the document's order, or
/// [`Error::NoItem`] where the list does not reach it.
fn one(config: &Config, index: usize) -> Result<&Daemon, Error> {
    match config.daemons.get_index(index) {
        Some((_, daemon)) => Ok(daemon),
        None => Err(Error::NoItem { list: "daemons", index, count: config.daemons.len() }),
    }
}

/// [`one`], to change it.
fn one_mut(config: &mut Config, index: usize) -> Result<&mut Daemon, Error> {
    let count = config.daemons.len();
    match config.daemons.get_index_mut(index) {
        Some((_, daemon)) => Ok(daemon),
        None => Err(Error::NoItem { list: "daemons", index, count }),
    }
}

/// `diverge cli default get|set`: which daemon is dialled when a
/// command names none. The document names it; here it is said by its
/// index among the daemons, as `daemons get` prints them:
///
/// - `get` prints `{"index":…,"name":"…"}`, or `null` where the
///   document's `default` names no daemon the document has.
/// - `set` takes an index and writes that daemon's name as the
///   default. An index the list does not reach is refused,
///   [`Error::NoItem`].
pub async fn default(root: &Path, action: Chosen) -> Result<(), Error> {
    match action {
        Chosen::Get => {
            let config = read::<Config>(root, Which::Cli).await?;
            let found = config.daemons.get_index_of(&config.default);
            line(&found.map(|index| serde_json::json!({"index": index, "name": config.default}))).await
        }
        Chosen::Set { index } => {
            set(root, |config| {
                let Some((name, _)) = config.daemons.get_index(index) else {
                    return Err(Error::NoItem { list: "daemons", index, count: config.daemons.len() });
                };
                config.default = name.clone();
                Ok(())
            })
            .await
        }
    }
}

/// The CLI's document read as its type, changed, and written whole.
/// A change that refuses writes nothing.
async fn set(root: &Path, change: impl FnOnce(&mut Config) -> Result<(), Error>) -> Result<(), Error> {
    let mut config = read::<Config>(root, Which::Cli).await?;
    change(&mut config)?;
    let yaml = serde_yaml_ng::to_string(&config).map_err(document::Error::Serialize)?;
    document::write(root, Which::Cli, &yaml).await?;
    Ok(())
}
