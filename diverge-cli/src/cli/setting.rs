//! One setting of the daemon's document — its port, its idle
//! seconds, its administrators, its identity, its Postgres — read
//! or written by itself.

use std::path::Path;

use tokio::io::AsyncWriteExt as _;

use diverge_daemon::config;
use diverge_daemon::config::Config;
use diverge_sdk::file_lock;
use diverge_sdk::shared;

use super::Error;
use crate::arguments::{Admins, Identity, IdleSeconds, Port, Postgres, PostgresKind};
use crate::document::{self, Which};
use crate::lifecycle::{self, Program};

/// `diverge daemon port get|set`: see [`get`] and [`set`].
pub async fn port(root: &Path, action: Port) -> Result<(), Error> {
    match action {
        Port::Get => get(root, |config| config.port.into()).await,
        Port::Set { value } => {
            set(root, |config| {
                config.port = value;
                Ok(())
            })
            .await
        }
    }
}

/// `diverge daemon idle_seconds get|set`: see [`get`] and [`set`].
pub async fn idle_seconds(root: &Path, action: IdleSeconds) -> Result<(), Error> {
    match action {
        IdleSeconds::Get => get(root, |config| config.idle_seconds).await,
        IdleSeconds::Set { value } => {
            set(root, |config| {
                config.idle_seconds = value;
                Ok(())
            })
            .await
        }
    }
}

/// `diverge daemon admins get|add|delete`, by these rules:
///
/// - A `get` prints every administrator, one JSON line each, in the
///   document's order, with its index before its issuer and subject:
///   `{"index":0,"issuer":"…","subject":"…"}`. No administrator, no
///   line. It is answered whether the daemon is running or not.
/// - An `add` puts the identity after the last administrator, so no
///   index already printed changes. An identity that is an
///   administrator already is refused with [`Error::AdminListed`],
///   and nothing is written.
/// - A `delete` removes the administrator at the index, and every one
///   after it moves one index down. An index the list does not reach
///   is refused with [`Error::NoAdmin`], and nothing is written.
/// - An `add` and a `delete` are each a [`set`]: refused while the
///   daemon runs on the root, and printing nothing.
pub async fn admins(root: &Path, action: Admins) -> Result<(), Error> {
    match action {
        Admins::Get => {
            let config = read(root).await?;
            let mut text = Vec::new();
            for (index, admin) in config.admins.iter().enumerate() {
                let line = serde_json::json!({"index": index, "issuer": admin.issuer, "subject": admin.subject});
                serde_json::to_writer(&mut text, &line).map_err(document::Error::Json)?;
                text.push(b'\n');
            }
            let mut stdout = tokio::io::stdout();
            stdout.write_all(&text).await.map_err(Error::Stdout)?;
            stdout.flush().await.map_err(Error::Stdout)
        }
        Admins::Add { issuer, subject } => {
            set(root, |config| {
                let admin = shared::Identity { issuer, subject };
                match config.admins.iter().position(|listed| *listed == admin) {
                    Some(index) => Err(Error::AdminListed { admin, index }),
                    None => {
                        config.admins.push(admin);
                        Ok(())
                    }
                }
            })
            .await
        }
        Admins::Delete { index } => {
            set(root, |config| {
                if index >= config.admins.len() {
                    return Err(Error::NoAdmin { index, count: config.admins.len() });
                }
                config.admins.remove(index);
                Ok(())
            })
            .await
        }
    }
}

/// `diverge daemon identity get|set`, by these rules:
///
/// - A `get` prints the identity as one JSON line,
///   `{"issuer":"…","subject":"…"}`, or `null` where the document has
///   none. It is answered whether the daemon is running or not.
/// - A `set` takes both the issuer and the subject and writes the
///   identity whole, replacing the one there. The document's
///   `identity` may be absent, and no `set` makes it so: an identity
///   is taken out of the file by hand.
/// - A `set` is a [`set`]: refused while the daemon runs on the root,
///   and printing nothing. Whether the identity is signed in on the
///   root is not asked here; the daemon finds that at its start.
pub async fn identity(root: &Path, action: Identity) -> Result<(), Error> {
    match action {
        Identity::Get => {
            let config = read(root).await?;
            let mut text = serde_json::to_vec(&config.identity).map_err(document::Error::Json)?;
            text.push(b'\n');
            let mut stdout = tokio::io::stdout();
            stdout.write_all(&text).await.map_err(Error::Stdout)?;
            stdout.flush().await.map_err(Error::Stdout)
        }
        Identity::Set { issuer, subject } => {
            set(root, |config| {
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
///   the password itself is never printed. It is answered whether the
///   daemon is running or not.
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
/// - A `set` is a [`set`]: refused while the daemon runs on the root,
///   and printing nothing.
pub async fn postgres(root: &Path, action: Postgres) -> Result<(), Error> {
    match action {
        Postgres::Get => {
            let config = read(root).await?;
            let line = match &config.postgres {
                config::Postgres::Local { max_connections } => {
                    serde_json::json!({"kind": "local", "max_connections": max_connections})
                }
                config::Postgres::Remote { url, password } => {
                    serde_json::json!({"kind": "remote", "url": url, "has_password": password.is_some()})
                }
            };
            let mut text = serde_json::to_vec(&line).map_err(document::Error::Json)?;
            text.push(b'\n');
            let mut stdout = tokio::io::stdout();
            stdout.write_all(&text).await.map_err(Error::Stdout)?;
            stdout.flush().await.map_err(Error::Stdout)
        }
        Postgres::Set(kind) => {
            set(root, |config| {
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

/// The daemon's document at `root`, as the daemon's own type takes
/// it: every setting the file leaves out at its default, and the
/// built-in defaults where there is no file.
async fn read(root: &Path) -> Result<Config, Error> {
    let value = document::read(root, Which::Daemon).await?;
    Ok(serde_path_to_error::deserialize(value).map_err(document::Error::Invalid)?)
}

/// One number of the daemon's document printed, alone on its line:
/// the value the daemon would run with, whether the file states it or
/// not. A `get` is answered whether the daemon is running or not.
async fn get(root: &Path, of: impl FnOnce(&Config) -> u64) -> Result<(), Error> {
    let config = read(root).await?;
    let mut stdout = tokio::io::stdout();
    stdout.write_all(format!("{}\n", of(&config)).as_bytes()).await.map_err(Error::Stdout)?;
    stdout.flush().await.map_err(Error::Stdout)
}

/// One setting of the daemon's document written, by these rules:
///
/// - The root's lock is probed first — [`file_lock::try_exclusive`],
///   as a start probes it. A lock that is held is the daemon running
///   on the root, and the set is refused with [`Error::Stop`], naming
///   the process the holder left in the lock file: a running daemon
///   has read its document already, and a setting written under it
///   would be one it does not run with. Nothing is written.
/// - The probe is kept until the document is written, so that no
///   daemon starts on the root between the two.
/// - The document is read as the daemon's type, the setting changed,
///   and the whole written as that type's YAML: see
///   [`document`](crate::document) for what a rewrite costs. A change
///   that refuses — an index the list does not reach, an identity
///   already listed — writes nothing.
/// - Nothing is printed.
async fn set(root: &Path, change: impl FnOnce(&mut Config) -> Result<(), Error>) -> Result<(), Error> {
    let program = Program::Daemon;
    let lock = program.lock(root);
    let Some(probe) = file_lock::try_exclusive(&lock).await.map_err(lifecycle::Error::Lock)? else {
        let holder = file_lock::holder(&lock).await.map_err(lifecycle::Error::Lock)?;
        return Err(Error::Stop { program, holder });
    };
    let mut config = read(root).await?;
    change(&mut config)?;
    let yaml = serde_yaml_ng::to_string(&config).map_err(document::Error::Serialize)?;
    document::write(root, Which::Daemon, &yaml).await?;
    drop(probe);
    Ok(())
}
