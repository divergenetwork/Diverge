//! The podman machine under the old path: stopped, and the files that
//! name where it is.

use std::path::{Path, PathBuf};

use super::Error;
use crate::host::tools::podman;

/// Where podman keeps its machines beneath a home of its own.
const MACHINES: [&str; 3] = ["containers", "podman", "machine"];

/// The provider's machine under `old` stopped, if it runs: rule 4 of
/// [`relocate`](super). A provider that stops stops its machine, and
/// one that was killed does not; and a machine may be started by
/// hand. Whoever left it running, its disk is held open while it
/// runs, and is not moved.
///
/// - Podman is told that `old` is its home, as the provider tells it,
///   so the machine it lists is the one kept there. Nothing is asked
///   of podman where `old` holds no machines' folder at all: there is
///   no machine, and podman need not even be installed.
/// - On Windows podman lists the machines of every home, so one
///   listed whose SSH key is not under `old` is another provider's,
///   and is left alone.
pub(super) async fn still(old: &Path) -> Result<(), Error> {
    let machines = MACHINES.iter().fold(old.to_path_buf(), |path, part| path.join(part));
    if tokio::fs::metadata(&machines).await.is_err() {
        return Ok(());
    }
    podman::configure(old.to_path_buf());
    let Some(machine) = podman::machine().await? else {
        return Ok(());
    };
    if machine.state != "running" || !under(&machine.ssh.identity, old) {
        return Ok(());
    }
    Ok(podman::podman(["machine", "stop", podman::MACHINE])
        .await?
        .require("podman", |status| status.success())?)
}

/// Whether `path` is `base` or lies under it, compared as the host's
/// filesystem compares names: without regard to the case of ASCII
/// letters on Windows and macOS, exactly elsewhere. A `\\?\` before a
/// Windows path is not part of it.
pub fn under(path: &Path, base: &Path) -> bool {
    let plain = |path: &Path| {
        let text = path.to_string_lossy();
        let text = text.strip_prefix(r"\\?\").unwrap_or(&text).replace('\\', "/");
        let text = text.trim_end_matches('/').to_string();
        if cfg!(any(windows, target_os = "macos")) { text.to_ascii_lowercase() } else { text }
    };
    let (path, base) = (plain(path), plain(base));
    path == base || path.strip_prefix(&base).is_some_and(|rest| rest.starts_with('/'))
}

/// Every file that names, by absolute path, something kept under the
/// storage path — which is `new` by the time this is asked, on macOS,
/// where the files have moved with the rest:
///
/// - **macOS.** Podman's configuration home is the storage path, so
///   they are under it: each machine's description,
///   `containers/podman/machine/<provider>/<name>.json`, and the
///   connections, `containers/podman-connections.json`.
/// - **Windows.** Podman takes no configuration home there. The
///   descriptions are under the user's `.config`,
///   `containers\podman\machine\<provider>\<name>.json`, and the
///   connections are `PODMAN_CONNECTIONS_CONF` where that is set,
///   else `%APPDATA%\containers\podman-connections.json`.
///
/// Every description is listed, of every machine and provider kind:
/// a machine another hand made with its data under the same path is
/// carried with it, and its description names the path as ours does.
/// A file or a folder that is not there is not listed.
pub(super) async fn references(new: &Path) -> Result<Vec<PathBuf>, Error> {
    let (config, connections) = homes(new);
    let mut files = Vec::new();
    if let Some(config) = config {
        let machines = MACHINES.iter().fold(config, |path, part| path.join(part));
        for kind in folders(&machines).await? {
            let mut listed = match tokio::fs::read_dir(&kind).await {
                Ok(listed) => listed,
                Err(_) => continue,
            };
            while let Some(entry) = listed.next_entry().await.map_err(|source| Error::io(&kind, source))? {
                let path = entry.path();
                if path.extension().is_some_and(|extension| extension == "json") {
                    files.push(path);
                }
            }
        }
    }
    if let Some(connections) = connections {
        if tokio::fs::metadata(&connections).await.is_ok() {
            files.push(connections);
        }
    }
    Ok(files)
}

/// Podman's configuration home, and its connections file, on this
/// host: see [`references`].
fn homes(new: &Path) -> (Option<PathBuf>, Option<PathBuf>) {
    if cfg!(windows) {
        let config = std::env::var_os("USERPROFILE").map(|home| PathBuf::from(home).join(".config"));
        let connections = std::env::var_os("PODMAN_CONNECTIONS_CONF").map(PathBuf::from).or_else(|| {
            std::env::var_os("APPDATA").map(|data| PathBuf::from(data).join("containers").join("podman-connections.json"))
        });
        (config, connections)
    } else {
        (Some(new.to_path_buf()), Some(new.join("containers").join("podman-connections.json")))
    }
}

/// The folders in `path`; none where it is not there.
async fn folders(path: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut listed = match tokio::fs::read_dir(path).await {
        Ok(listed) => listed,
        Err(_) => return Ok(Vec::new()),
    };
    let mut folders = Vec::new();
    while let Some(entry) = listed.next_entry().await.map_err(|source| Error::io(path, source))? {
        if entry.file_type().await.is_ok_and(|kind| kind.is_dir()) {
            folders.push(entry.path());
        }
    }
    Ok(folders)
}

/// On Windows, asked to leave the data behind: whether podman still
/// holds a description of the provider's machine that names the old
/// path. It would: the descriptions are the user's and not the
/// path's, so the machine is listed whatever the storage path is, and
/// the provider refuses to start beside a machine whose SSH key is not
/// under its storage path. [`Error::Blocked`] where it does. Read from
/// the description itself; podman is not asked.
#[cfg(windows)]
pub async fn blocked(old: &Path) -> Result<(), Error> {
    let olds = [super::rewrite::written(old)?];
    for file in references(old).await? {
        if file.file_stem().is_none_or(|name| name != podman::MACHINE) {
            continue;
        }
        let Ok(text) = tokio::fs::read_to_string(&file).await else {
            continue;
        };
        if super::rewrite::names(&text, &olds) {
            return Err(Error::Blocked(old.to_path_buf()));
        }
    }
    Ok(())
}
