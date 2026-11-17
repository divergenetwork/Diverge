//! One setting of the viewer's document, read or written by itself:
//! the daemon it dials, and as whom. The only way the CLI works on
//! `viewer.yaml`: there is no `viewer config`.

use std::path::Path;

use diverge_sdk::shared;
use diverge_viewer_config::Config;

use super::Error;
use super::setting::{line, read, write};
use crate::arguments::{Identity, RedialSetting, Text, ViewerDaemon};
use crate::document::{self, Which};

/// `diverge viewer daemon …`: the settings of the viewer's `daemon`,
/// by these rules:
///
/// - Every setting has a default — the address of a daemon on this
///   host at its default port, an issuer and a subject that are
///   empty, no identity for the daemon, a redial of five seconds — so
///   every `get` answers and every `set` writes, whether the document
///   names a daemon, has a `daemon` with some settings left out, or
///   is not there.
/// - `address`, `issuer` and `subject` each print the string as it
///   is, alone on its line, and write the string given. An issuer or
///   a subject that is empty is the viewer naming no identity, and
///   the app does not start until both are set.
/// - `identity get` prints `{"issuer":"…","subject":"…"}`, or `null`
///   where none is stated; `identity set` takes both and writes the
///   identity whole. No `set` makes it absent: it is taken out of the
///   file by hand.
/// - `redial get` prints `never` or the number of seconds; `redial
///   set` takes either.
/// - No lock is taken: the viewer holds none on the root. A setting
///   written while the app is open is read at the app's next start.
/// - A `set` rewrites the file whole, as the viewer's own type writes
///   it: see [`document`] for what that costs. Nothing is printed.
pub async fn daemon(root: &Path, action: ViewerDaemon) -> Result<(), Error> {
    match action {
        ViewerDaemon::Address(action) => text(root, action, |config| &mut config.daemon.address).await,
        ViewerDaemon::Issuer(action) => text(root, action, |config| &mut config.daemon.issuer).await,
        ViewerDaemon::Subject(action) => text(root, action, |config| &mut config.daemon.subject).await,
        ViewerDaemon::Identity(Identity::Get) => line(&read::<Config>(root, Which::Viewer).await?.daemon.identity).await,
        ViewerDaemon::Identity(Identity::Set { issuer, subject }) => {
            set(root, |config| config.daemon.identity = Some(shared::Identity { issuer, subject })).await
        }
        ViewerDaemon::Redial(RedialSetting::Get) => {
            let redial = read::<Config>(root, Which::Viewer).await?.daemon.redial;
            let said = match serde_json::to_value(redial).map_err(document::Error::Json)? {
                serde_json::Value::String(word) => word,
                seconds => seconds.to_string(),
            };
            write(format!("{said}\n").into_bytes()).await
        }
        ViewerDaemon::Redial(RedialSetting::Set { value }) => set(root, |config| config.daemon.redial = value).await,
    }
}

/// One string of the document read or written: `of` is the string in
/// it.
async fn text(root: &Path, action: Text, of: impl FnOnce(&mut Config) -> &mut String) -> Result<(), Error> {
    match action {
        Text::Get => {
            let mut config = read::<Config>(root, Which::Viewer).await?;
            write(format!("{}\n", of(&mut config)).into_bytes()).await
        }
        Text::Set { value } => set(root, |config| *of(config) = value).await,
    }
}

/// The viewer's document read as its type, changed, and written
/// whole.
async fn set(root: &Path, change: impl FnOnce(&mut Config)) -> Result<(), Error> {
    let mut config = read::<Config>(root, Which::Viewer).await?;
    change(&mut config);
    let yaml = serde_yaml_ng::to_string(&config).map_err(document::Error::Serialize)?;
    document::write(root, Which::Viewer, &yaml).await?;
    Ok(())
}
