//! One setting of the daemon's document, read or written by itself.

use std::path::Path;

use tokio::io::AsyncWriteExt as _;

use diverge_daemon::config::Config;
use diverge_sdk::file_lock;

use super::Error;
use crate::arguments::{IdleSeconds, Port};
use crate::document::{self, Which};
use crate::lifecycle::{self, Program};

/// `diverge daemon port get|set`: see [`get`] and [`set`].
pub async fn port(root: &Path, action: Port) -> Result<(), Error> {
    match action {
        Port::Get => get(root, |config| config.port.into()).await,
        Port::Set { value } => set(root, |config| config.port = value).await,
    }
}

/// `diverge daemon idle_seconds get|set`: see [`get`] and [`set`].
pub async fn idle_seconds(root: &Path, action: IdleSeconds) -> Result<(), Error> {
    match action {
        IdleSeconds::Get => get(root, |config| config.idle_seconds).await,
        IdleSeconds::Set { value } => set(root, |config| config.idle_seconds = value).await,
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
///   [`document`](crate::document) for what a rewrite costs.
/// - Nothing is printed.
async fn set(root: &Path, change: impl FnOnce(&mut Config)) -> Result<(), Error> {
    let program = Program::Daemon;
    let lock = program.lock(root);
    let Some(probe) = file_lock::try_exclusive(&lock).await.map_err(lifecycle::Error::Lock)? else {
        let holder = file_lock::holder(&lock).await.map_err(lifecycle::Error::Lock)?;
        return Err(Error::Stop { program, holder });
    };
    let mut config = read(root).await?;
    change(&mut config);
    let yaml = serde_yaml_ng::to_string(&config).map_err(document::Error::Serialize)?;
    document::write(root, Which::Daemon, &yaml).await?;
    drop(probe);
    Ok(())
}
