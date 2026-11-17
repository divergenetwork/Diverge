//! Finding the root.

use std::path::PathBuf;

use super::Error;

/// The root, from `given` — the `--config` directory, if one was —
/// else the environment, else the home directory; made if absent,
/// and made absolute.
///
/// With no `--config`, `DIVERGE_CONFIG` names the root; with
/// neither, it is `.diverge` under the home directory — `HOME`, or
/// `USERPROFILE` on Windows — and a host with no home is
/// [`Error::Home`]. The path is made absolute against the working
/// directory without following links, so what the app names by it
/// is the path as it was given.
pub async fn root(given: Option<PathBuf>) -> Result<PathBuf, Error> {
    let root = match given.or_else(|| std::env::var_os("DIVERGE_CONFIG").map(PathBuf::from)) {
        Some(root) => root,
        None => home()?.join(".diverge"),
    };
    let root = std::path::absolute(&root).map_err(|source| Error::Io {
        path: root.clone(),
        source,
    })?;
    tokio::fs::create_dir_all(&root).await.map_err(|source| Error::Io {
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
