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

/// Everything outside the storage path `path` that names it,
/// removed: what a delete of the path does before the folder goes —
/// see [`wipe`](super::wipe). On macOS there is nothing outside it,
/// and this does nothing. On Windows, in this order:
///
/// 1. Each machine whose description names the path is one podman
///    is asked to remove, `podman machine rm --force <name>`: podman
///    unregisters the machine's WSL distribution, deletes its
///    description and takes out its connections. What podman does
///    not do — it is missing, or the machine is half there and it
///    refuses — is not the error, since the steps after do the same
///    by hand.
/// 2. Every WSL distribution whose disk is under the path is
///    unregistered: see [`unregister`](super::wsl::unregister).
/// 3. Each description that still names the path is deleted, with
///    the ignition file and the lock beside it.
/// 4. Each connection whose SSH key is under the path is taken out
///    of the connections file, and the default with it where it was
///    one of them.
/// 5. No file left names the path, or it is [`Error::Remains`].
pub(super) async fn forget(path: &Path) -> Result<(), Error> {
    if !cfg!(windows) {
        return Ok(());
    }
    let olds = [super::rewrite::written(path)?];
    let (descriptions, connections) = named(path, &olds).await?;
    if !descriptions.is_empty() {
        podman::configure(path.to_path_buf());
        for description in &descriptions {
            if let Some(name) = description.file_stem().and_then(|name| name.to_str()) {
                let _ = podman::podman(["machine", "rm", "--force", name]).await;
            }
        }
    }
    #[cfg(windows)]
    super::wsl::unregister(path).await?;
    let (descriptions, connections_left) = named(path, &olds).await?;
    for description in descriptions {
        for extension in ["json", "ign", "lock"] {
            super::moving::remove(&description.with_extension(extension)).await?;
        }
    }
    if let Some(file) = connections_left.or(connections) {
        disconnect(&file, path).await?;
    }
    let (descriptions, connections) = named(path, &olds).await?;
    match descriptions.into_iter().next().or(connections) {
        Some(file) => Err(Error::Remains(file)),
        None => Ok(()),
    }
}

/// The files of [`references`] that name `path`: the machines'
/// descriptions, and the connections file where it does.
async fn named(path: &Path, olds: &[String]) -> Result<(Vec<PathBuf>, Option<PathBuf>), Error> {
    let mut descriptions = Vec::new();
    let mut connections = None;
    for file in references(path).await? {
        let Ok(text) = tokio::fs::read_to_string(&file).await else {
            continue;
        };
        if !super::rewrite::names(&text, olds) {
            continue;
        }
        if file.file_name().is_some_and(|name| name == "podman-connections.json") {
            connections = Some(file);
        } else {
            descriptions.push(file);
        }
    }
    Ok((descriptions, connections))
}

/// The connections file at `file` with every connection whose SSH
/// key is under `path` taken out: `Connection.Connections` is a map
/// of name to `{URI, Identity, IsMachine}`, and `Connection.Default`
/// names one of them, or is empty. A default that was taken out is
/// made empty, as podman leaves it. A file with no such connection is
/// not written; one that is not that JSON is [`Error::Connections`].
async fn disconnect(file: &Path, path: &Path) -> Result<(), Error> {
    let text = tokio::fs::read_to_string(file).await.map_err(|source| Error::io(file, source))?;
    let mut document: serde_json::Value = serde_json::from_str(&text).map_err(|_| Error::Connections(file.to_path_buf()))?;
    let Some(connection) = document.get_mut("Connection").and_then(serde_json::Value::as_object_mut) else {
        return Err(Error::Connections(file.to_path_buf()));
    };
    let mut gone = Vec::new();
    if let Some(connections) = connection.get_mut("Connections").and_then(serde_json::Value::as_object_mut) {
        connections.retain(|name, held| {
            let keep = !held.get("Identity").and_then(serde_json::Value::as_str).is_some_and(|key| under(Path::new(key), path));
            if !keep {
                gone.push(name.clone());
            }
            keep
        });
    }
    if gone.is_empty() {
        return Ok(());
    }
    let default_gone = connection.get("Default").and_then(serde_json::Value::as_str).is_some_and(|name| gone.iter().any(|gone| gone == name));
    if default_gone {
        connection.insert("Default".to_string(), serde_json::Value::String(String::new()));
    }
    let written = serde_json::to_string_pretty(&document).map_err(|_| Error::Connections(file.to_path_buf()))?;
    let mut beside = file.as_os_str().to_owned();
    beside.push(".diverge-tmp");
    let beside = PathBuf::from(beside);
    tokio::fs::write(&beside, written).await.map_err(|source| Error::io(&beside, source))?;
    tokio::fs::rename(&beside, file).await.map_err(|source| Error::io(file, source))
}
