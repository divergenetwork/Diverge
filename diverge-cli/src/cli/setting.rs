//! One setting of a program's document, read or written by itself:
//! what every such command shares, and the daemon's own — its port,
//! its idle seconds, its administrators, its identity, its Postgres.
//! The provider's are [`provider`](super::provider)'s.
//!
//! # The rules every setting keeps
//!
//! - A read — a `get` — is answered whether the program is running
//!   or not, with the value the program would run with: the default
//!   where the file leaves the setting out, and the built-in defaults
//!   where there is no file.
//! - A write — a `set`, an `add`, a `delete` — is refused while the
//!   program runs on the root: see [`set`].
//! - A write prints nothing.
//! - Nothing is given as JSON: a setting's members are arguments and
//!   flags.
//! - A list is printed one JSON line per item, each with its `index`
//!   first; an `add` puts the item after the last, and refuses one
//!   the list already has; a `delete` takes the index, and refuses one
//!   the list does not reach.

use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::AsyncWriteExt as _;

use diverge_daemon::config;
use diverge_daemon::config::Config;
use diverge_sdk::file_lock;
use diverge_sdk::shared;

use super::Error;
use crate::arguments::{Admins, Identity, IdleSeconds, Port, Postgres, PostgresKind};
use crate::document::{self, Which};
use crate::lifecycle::{self, Program};

/// `diverge daemon port get|set`: see [`number`] and [`set`].
pub async fn port(root: &Path, action: Port) -> Result<(), Error> {
    match action {
        Port::Get => number(read::<Config>(root, Which::Daemon).await?.port.into()).await,
        Port::Set { value } => {
            daemon(root, |config| {
                config.port = value;
                Ok(())
            })
            .await
        }
    }
}

/// `diverge daemon idle_seconds get|set`: see [`number`] and [`set`].
pub async fn idle_seconds(root: &Path, action: IdleSeconds) -> Result<(), Error> {
    match action {
        IdleSeconds::Get => number(read::<Config>(root, Which::Daemon).await?.idle_seconds).await,
        IdleSeconds::Set { value } => {
            daemon(root, |config| {
                config.idle_seconds = value;
                Ok(())
            })
            .await
        }
    }
}

/// `diverge daemon admins get|add|delete`: the list of identities
/// `admins`, by the rules of [`identities`], [`add`] and [`delete`].
pub async fn admins(root: &Path, action: Admins) -> Result<(), Error> {
    match action {
        Admins::Get => identities(&read::<Config>(root, Which::Daemon).await?.admins).await,
        Admins::Add { issuer, subject } => {
            daemon(root, |config| add("admins", &mut config.admins, shared::Identity { issuer, subject })).await
        }
        Admins::Delete { index } => daemon(root, |config| delete("admins", &mut config.admins, index)).await,
    }
}

/// `diverge daemon identity get|set`, by these rules:
///
/// - A `get` prints the identity as one JSON line,
///   `{"issuer":"…","subject":"…"}`, or `null` where the document has
///   none.
/// - A `set` takes both the issuer and the subject and writes the
///   identity whole, replacing the one there. The document's
///   `identity` may be absent, and no `set` makes it so: an identity
///   is taken out of the file by hand.
/// - Whether the identity is signed in on the root is not asked here;
///   the daemon finds that at its start.
pub async fn identity(root: &Path, action: Identity) -> Result<(), Error> {
    match action {
        Identity::Get => line(&read::<Config>(root, Which::Daemon).await?.identity).await,
        Identity::Set { issuer, subject } => {
            daemon(root, |config| {
                config.identity = Some(shared::Identity { issuer, subject });
                Ok(())
            })
            .await
        }
    }
}

/// `diverge daemon postgres get|set`, by these rules:
///
/// - A `get` prints which database as one JSON line: for the local
///   kind `{"kind":"local","max_connections":…}`; for the remote kind
///   `{"kind":"remote","url":"…","has_password":…}`, where
///   `has_password` says whether the document holds a password and
///   the password itself is never printed.
/// - A `set` names the kind as its own command — `set local`,
///   `set remote` — and takes that kind's settings as flags, so the
///   command line cannot give one kind the other's.
/// - A `set` writes `postgres` whole, and carries nothing over from
///   the document: `set local` without `--max-connections` writes the
///   default, whatever the document had; `set remote` without
///   `--password` writes no password, whatever the document had.
/// - `set remote` requires `--url`. The URL is written as given, and
///   is not dialled or parsed here; the daemon finds a wrong one at
///   its start.
pub async fn postgres(root: &Path, action: Postgres) -> Result<(), Error> {
    match action {
        Postgres::Get => {
            let config = read::<Config>(root, Which::Daemon).await?;
            line(&match &config.postgres {
                config::Postgres::Local { max_connections } => {
                    serde_json::json!({"kind": "local", "max_connections": max_connections})
                }
                config::Postgres::Remote { url, password } => {
                    serde_json::json!({"kind": "remote", "url": url, "has_password": password.is_some()})
                }
            })
            .await
        }
        Postgres::Set(kind) => {
            daemon(root, |config| {
                config.postgres = match kind {
                    PostgresKind::Local { max_connections: Some(max_connections) } => {
                        config::Postgres::Local { max_connections }
                    }
                    PostgresKind::Local { max_connections: None } => config::Postgres::default(),
                    PostgresKind::Remote { url, password } => config::Postgres::Remote { url, password },
                };
                Ok(())
            })
            .await
        }
    }
}

/// A write to the daemon's document: see [`set`].
async fn daemon(root: &Path, change: impl FnOnce(&mut Config) -> Result<(), Error>) -> Result<(), Error> {
    set(root, Program::Daemon, Which::Daemon, change).await
}

/// The document `which` names at `root`, as its program's own type
/// `T` takes it: every setting the file leaves out at its default,
/// and the built-in defaults where there is no file.
pub(super) async fn read<T: DeserializeOwned>(root: &Path, which: Which) -> Result<T, Error> {
    let value = document::read(root, which).await?;
    Ok(serde_path_to_error::deserialize(value).map_err(document::Error::Invalid)?)
}

/// One number printed, alone on its line.
pub(super) async fn number(value: u64) -> Result<(), Error> {
    write(format!("{value}\n").into_bytes()).await
}

/// One value printed as JSON, alone on its line.
pub(super) async fn line(value: &impl Serialize) -> Result<(), Error> {
    let mut text = serde_json::to_vec(value).map_err(document::Error::Json)?;
    text.push(b'\n');
    write(text).await
}

/// A list of identities printed, one JSON line each, in the
/// document's order, with its index before its issuer and subject:
/// `{"index":0,"issuer":"…","subject":"…"}`. No item, no line.
pub(super) async fn identities(list: &[shared::Identity]) -> Result<(), Error> {
    let mut text = Vec::new();
    for (index, identity) in list.iter().enumerate() {
        let line = serde_json::json!({"index": index, "issuer": identity.issuer, "subject": identity.subject});
        serde_json::to_writer(&mut text, &line).map_err(document::Error::Json)?;
        text.push(b'\n');
    }
    write(text).await
}

/// `text` on stdout, flushed.
pub(super) async fn write(text: Vec<u8>) -> Result<(), Error> {
    let mut stdout = tokio::io::stdout();
    stdout.write_all(&text).await.map_err(Error::Stdout)?;
    stdout.flush().await.map_err(Error::Stdout)
}

/// `item` put after the last of the list named `name`, so that no
/// index already printed changes. An item the list already has is
/// refused with [`Error::Listed`], naming where it is.
pub(super) fn add<T: PartialEq + std::fmt::Display>(name: &'static str, list: &mut Vec<T>, item: T) -> Result<(), Error> {
    match list.iter().position(|listed| *listed == item) {
        Some(index) => Err(Error::Listed { list: name, item: item.to_string(), index }),
        None => {
            list.push(item);
            Ok(())
        }
    }
}

/// The item at `index` taken out of the list named `name`; every item
/// after it moves one index down. An index the list does not reach is
/// refused with [`Error::NoItem`].
pub(super) fn delete<T>(name: &'static str, list: &mut Vec<T>, index: usize) -> Result<(), Error> {
    if index >= list.len() {
        return Err(Error::NoItem { list: name, index, count: list.len() });
    }
    list.remove(index);
    Ok(())
}

/// `program`'s lock on the root, taken: what every write of its
/// document holds for its whole length, so that the program neither
/// runs under the write nor starts in the middle of it. A lock that
/// is held is the program running, and is [`Error::Stop`], naming the
/// process the holder left in the lock file. The lock is let go when
/// what this answers is dropped.
pub(super) async fn hold(root: &Path, program: Program) -> Result<file_lock::Held, Error> {
    let lock = program.lock(root);
    match file_lock::try_exclusive(&lock).await.map_err(lifecycle::Error::Lock)? {
        Some(held) => Ok(held),
        None => {
            let holder = file_lock::holder(&lock).await.map_err(lifecycle::Error::Lock)?;
            Err(Error::Stop { program, holder })
        }
    }
}

/// One setting of `program`'s document written, by these rules:
///
/// - The root's lock is taken first — see [`hold`]. A lock that is
///   held is the program running on the root, and the write is
///   refused with [`Error::Stop`]: a running program has read its
///   document already, and a setting written under it would be one
///   it does not run with. Nothing is written.
/// - The lock is kept until the document is written, so that the
///   program does not start on the root between the two.
/// - The document is read as the program's type, the setting changed,
///   and the whole written as that type's YAML: see
///   [`document`](crate::document) for what a rewrite costs. A change
///   that refuses — an index the list does not reach, an item already
///   listed — writes nothing.
/// - Nothing is printed.
pub(super) async fn set<T: DeserializeOwned + Serialize>(
    root: &Path,
    program: Program,
    which: Which,
    change: impl FnOnce(&mut T) -> Result<(), Error>,
) -> Result<(), Error> {
    let probe = hold(root, program).await?;
    let mut config = read::<T>(root, which).await?;
    change(&mut config)?;
    let yaml = serde_yaml_ng::to_string(&config).map_err(document::Error::Serialize)?;
    document::write(root, which, &yaml).await?;
    drop(probe);
    Ok(())
}
