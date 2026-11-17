//! The viewer's own settings, each read or written by itself: the
//! port its stop is heard on, and its redial. Its daemons and its
//! default daemon are [`own`](super::own)'s, as the CLI's are.

use std::path::Path;

use super::Error;
use super::setting::{number, read, set, write};
use crate::arguments::{Port, RedialSetting};
use crate::config::Viewer;
use crate::document::{self, Which};
use crate::lifecycle::Program;

/// `diverge viewer port get|set`: the loopback port the viewer hears
/// its stop on. A `get` prints the number, the default where none
/// was set; a `set` is refused while the viewer runs on the root —
/// see [`set`].
pub async fn port(root: &Path, action: Port) -> Result<(), Error> {
    match action {
        Port::Get => number(read::<Viewer>(root, Which::Viewer).await?.port.into()).await,
        Port::Set { value } => {
            set(root, Program::Viewer, Which::Viewer, |config: &mut Viewer| {
                config.port = value;
                Ok(())
            })
            .await
        }
    }
}

/// `diverge viewer redial get|set`: `get` prints `never` or the
/// number of seconds; `set` takes either, and is refused while the
/// viewer runs on the root — see [`set`].
pub async fn redial(root: &Path, action: RedialSetting) -> Result<(), Error> {
    match action {
        RedialSetting::Get => {
            let redial = read::<Viewer>(root, Which::Viewer).await?.redial;
            let said = match serde_json::to_value(redial).map_err(document::Error::Json)? {
                serde_json::Value::String(word) => word,
                seconds => seconds.to_string(),
            };
            write(format!("{said}\n").into_bytes()).await
        }
        RedialSetting::Set { value } => {
            set(root, Program::Viewer, Which::Viewer, |config: &mut Viewer| {
                config.redial = value;
                Ok(())
            })
            .await
        }
    }
}
