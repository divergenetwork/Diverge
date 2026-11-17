//! The CLI's own document, a setting at a time: its daemons, and
//! which is the default.

use std::path::Path;

use indexmap::IndexMap;

use super::Error;
use super::setting::{line, read, write};
use crate::arguments::{Daemons, DefaultDaemon};
use crate::config::{Config, Daemon};
use crate::document::{self, Which};

/// `diverge cli daemons get|add|delete`: the daemons the CLI may
/// dial. The document keeps them by name, in an order; here they are a
/// list in that order, each with its name as a member, so that they
/// are worked on as every other list is. By these rules:
///
/// - `get` prints each daemon as one JSON line with its `index`
///   first, then its `name` and its `address`, and its `identity`
///   where the daemon's own is stated. A daemon's accounts are not
///   printed here.
/// - `add` puts the daemon after the last, with no account, no
///   default account and no identity of its own. A name that is a
///   daemon's already is refused, [`Error::Listed`]: the name is how a
///   daemon is asked for.
/// - `delete` takes the index, and removes the daemon with its
///   accounts. The default daemon is refused, [`Error::Default`]: a
///   document whose `default` names no daemon dials none, and which
///   to dial instead is not the delete's to choose.
/// - No lock is taken: the CLI holds none on the root, and reads the
///   document at every command. Nothing is printed by a write.
pub async fn daemons(root: &Path, action: Daemons) -> Result<(), Error> {
    match action {
        Daemons::Get => {
            let config = read::<Config>(root, Which::Cli).await?;
            let mut text = Vec::new();
            for (index, (name, daemon)) in config.daemons.iter().enumerate() {
                let mut line = serde_json::json!({"index": index, "name": name, "address": daemon.address});
                if let Some(identity) = &daemon.identity {
                    line["identity"] = serde_json::json!({"issuer": identity.issuer, "subject": identity.subject});
                }
                serde_json::to_writer(&mut text, &line).map_err(document::Error::Json)?;
                text.push(b'\n');
            }
            write(text).await
        }
        Daemons::Add { name, address } => {
            set(root, |config| {
                if let Some(index) = config.daemons.get_index_of(&name) {
                    return Err(Error::Listed { list: "daemons", item: format!("a daemon named `{name}`"), index });
                }
                config.daemons.insert(name, Daemon { address, default: None, identity: None, accounts: IndexMap::new() });
                Ok(())
            })
            .await
        }
        Daemons::Delete { index } => {
            set(root, |config| {
                let Some((name, _)) = config.daemons.get_index(index) else {
                    return Err(Error::NoItem { list: "daemons", index, count: config.daemons.len() });
                };
                if *name == config.default {
                    return Err(Error::Default { name: name.clone(), index });
                }
                // Shifted, not swapped: every daemon after it moves
                // one index down, and the order is otherwise kept.
                config.daemons.shift_remove_index(index);
                Ok(())
            })
            .await
        }
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
pub async fn default(root: &Path, action: DefaultDaemon) -> Result<(), Error> {
    match action {
        DefaultDaemon::Get => {
            let config = read::<Config>(root, Which::Cli).await?;
            let found = config.daemons.get_index_of(&config.default);
            line(&found.map(|index| serde_json::json!({"index": index, "name": config.default}))).await
        }
        DefaultDaemon::Set { index } => {
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
