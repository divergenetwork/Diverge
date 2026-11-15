//! Finding the root.

use std::path::PathBuf;

use super::Error;

/// The directory under the home directory.
const DIVERGE: &str = ".diverge";

/// The root, from `given` — the `--config` directory, if one was —
/// else the environment, else the home directory's `.diverge`; made
/// if absent, and made absolute.
///
/// It is what every Diverge program does: `DIVERGE_CONFIG` names the
/// root when nothing was given, and the home directory — `HOME`, or
/// `USERPROFILE` on Windows — holds it when neither did, a host with
/// no home being [`Error::Home`]. A `.diverge` in the working
/// directory is nothing to it: that folder is where an image's source
/// keeps what the image says of itself. The path is made absolute
/// against the working directory without following links. A directory
/// that cannot be made is [`Error::Io`].
pub async fn root(given: Option<PathBuf>) -> Result<PathBuf, Error> {
    let root = match given.or_else(|| std::env::var_os("DIVERGE_CONFIG").map(PathBuf::from)) {
        Some(root) => root,
        None => home()?.join(DIVERGE),
    };
    let root = std::path::absolute(&root).map_err(|source| Error::Io {
        path: root.clone(),
        source,
    })?;
    tokio::fs::create_dir_all(&root)
        .await
        .map_err(|source| Error::Io {
            path: root.clone(),
            source,
        })?;
    Ok(root)
}

/// The home directory: `HOME`, and on Windows `USERPROFILE` where
/// `HOME` is not set.
fn home() -> Result<PathBuf, Error> {
    if let Some(home) = std::env::var_os("HOME") {
        return Ok(PathBuf::from(home));
    }
    if cfg!(windows) {
        if let Some(home) = std::env::var_os("USERPROFILE") {
            return Ok(PathBuf::from(home));
        }
    }
    Err(Error::Home)
}
