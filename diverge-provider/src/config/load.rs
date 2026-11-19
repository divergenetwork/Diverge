//! Reading the file, and resolving what it names.

use std::io;
use std::path::{Path, PathBuf};

use super::Config;
use super::Error;

/// The name the file has, and the only one looked for.
const FILE: &str = "provider.yaml";

/// The configuration at `root`: `provider.yaml` read and parsed, or
/// the defaults where there is no such file, with every path in it
/// resolved against `root` — `podman.storage_path`, each
/// store's and each fixed volume's `path`, `volumes.ephemeral_fuse`'s,
/// and `images.path` — so
/// that what comes back holds every path absolute. A file that does
/// not parse, or names a key the document does not have, is
/// [`Error::Parse`], with the path to the value that was wrong.
/// Nothing on disk is made or checked here: that is the provider's,
/// at its start.
pub async fn load(root: &Path) -> Result<Config, Error> {
    let file = root.join(FILE);
    let mut config: Config = match tokio::fs::read(&file).await {
        Ok(bytes) => serde_path_to_error::deserialize(serde_yaml_ng::Deserializer::from_slice(&bytes))
            .map_err(|source| Error::Parse { path: file, source })?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => Config::default(),
        Err(source) => return Err(Error::Io { path: file, source }),
    };
    config.podman.storage_path = resolve(root, &config.podman.storage_path);
    if let Some(volumes) = &mut config.volumes {
        for store in volumes.stores.iter_mut() {
            store.path = resolve(root, &store.path);
        }
        for fixed in volumes.fixed.iter_mut() {
            fixed.path = resolve(root, &fixed.path);
        }
        if let Some(ephemeral_fuse) = &mut volumes.ephemeral_fuse {
            ephemeral_fuse.path = resolve(root, &ephemeral_fuse.path);
        }
    }
    if let Some(images) = &mut config.images {
        images.path = resolve(root, &images.path);
    }
    Ok(config)
}

/// A path of the document as the provider takes it: a relative one
/// joined onto `root`, the directory that holds `provider.yaml`; an
/// absolute one as written. What [`load`] does to every path, and
/// what anything that reads a path out of the document unloaded —
/// the CLI, before it moves podman's data — does to it, so that both
/// mean the same directory.
pub fn resolve(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() { path.to_path_buf() } else { root.join(path) }
}
