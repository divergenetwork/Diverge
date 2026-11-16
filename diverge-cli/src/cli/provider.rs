//! One setting of the provider's document, read or written by itself:
//! its port, its identity, whom it accepts, whom it dials, the
//! registries it pulls from, and where podman keeps its data. Every
//! one keeps the rules of [`setting`](super::setting).

use std::path::{Path, PathBuf};

use diverge_provider::config::Config;
use diverge_provider::config::outgoing;
use diverge_provider::host::relocate;
use diverge_sdk::shared;

use super::Error;
use super::setting::{add, delete, hold, identities, line, number, read, set, write};
use crate::arguments::{Identity, Incoming, Outgoing, Podman, Port, Registries, StoragePath};
use crate::document::{self, Which};
use crate::lifecycle::Program;

/// `diverge provider port get|set`.
pub async fn port(root: &Path, action: Port) -> Result<(), Error> {
    match action {
        Port::Get => number(read::<Config>(root, Which::Provider).await?.port.into()).await,
        Port::Set { value } => {
            provider(root, |config| {
                config.port = value;
                Ok(())
            })
            .await
        }
    }
}

/// `diverge provider identity get|set`, by the rules of the daemon's
/// [`identity`](super::setting::identity): a `get` prints
/// `{"issuer":"…","subject":"…"}` or `null`; a `set` takes both the
/// issuer and the subject, writes the identity whole, and never makes
/// it absent.
pub async fn identity(root: &Path, action: Identity) -> Result<(), Error> {
    match action {
        Identity::Get => line(&read::<Config>(root, Which::Provider).await?.identity).await,
        Identity::Set { issuer, subject } => {
            provider(root, |config| {
                config.identity = Some(shared::Identity { issuer, subject });
                Ok(())
            })
            .await
        }
    }
}

/// `diverge provider incoming get|add|delete`: the list of identities
/// the provider accepts, `incoming`, as the daemon's `admins` is
/// worked on.
pub async fn incoming(root: &Path, action: Incoming) -> Result<(), Error> {
    match action {
        Incoming::Get => identities(&read::<Config>(root, Which::Provider).await?.incoming).await,
        Incoming::Add { issuer, subject } => {
            provider(root, |config| add("incoming", &mut config.incoming, shared::Identity { issuer, subject })).await
        }
        Incoming::Delete { index } => provider(root, |config| delete("incoming", &mut config.incoming, index)).await,
    }
}

/// `diverge provider outgoing get|add|delete`: the list of peers the
/// provider dials, `outgoing`, by these rules:
///
/// - A `get` prints every peer, one JSON line each, in the document's
///   order: `{"index":0,"address":"…","issuer":"…","subject":"…"}`,
///   the identity's two members beside the address and not beneath a
///   member of their own, as an `add` takes them.
/// - An `add` takes the address and the identity's issuer and
///   subject, all three, and puts the peer after the last. A peer the
///   list already has — the same address and the same identity — is
///   refused; the same identity at another address, or another
///   identity at the same address, is another peer.
/// - A `delete` takes the index.
pub async fn outgoing(root: &Path, action: Outgoing) -> Result<(), Error> {
    match action {
        Outgoing::Get => {
            let config = read::<Config>(root, Which::Provider).await?;
            let mut text = Vec::new();
            for (index, peer) in config.outgoing.iter().enumerate() {
                let line = serde_json::json!({
                    "index": index,
                    "address": peer.address,
                    "issuer": peer.identity.issuer,
                    "subject": peer.identity.subject,
                });
                serde_json::to_writer(&mut text, &line).map_err(document::Error::Json)?;
                text.push(b'\n');
            }
            write(text).await
        }
        Outgoing::Add { address, issuer, subject } => {
            provider(root, |config| {
                let identity = shared::Identity { issuer, subject };
                match config.outgoing.iter().position(|peer| peer.address == address && peer.identity == identity) {
                    Some(index) => {
                        Err(Error::Listed { list: "outgoing", item: format!("{identity}, at `{address}`,"), index })
                    }
                    None => {
                        config.outgoing.push(outgoing::Outgoing { address, identity });
                        Ok(())
                    }
                }
            })
            .await
        }
        Outgoing::Delete { index } => provider(root, |config| delete("outgoing", &mut config.outgoing, index)).await,
    }
}

/// `diverge provider podman …`: the settings of the `podman` section,
/// each named as the section names it.
pub async fn podman(root: &Path, action: Podman) -> Result<(), Error> {
    match action {
        Podman::Registries(action) => registries(root, action).await,
        Podman::StoragePath(action) => storage_path(root, action).await,
    }
}

/// `diverge provider podman registries get|add|delete`: the list of
/// registries the provider pulls from, `podman.registries`, by these
/// rules:
///
/// - A `get` prints every registry, one JSON line each, in the
///   document's order: `{"index":0,"host":"docker.io"}`. A document
///   with no `podman` section, or none of its `registries`, has the
///   default three, and those are what is printed.
/// - An `add` takes the host and puts it after the last, and refuses
///   one listed already. The host is written as given: that it is a
///   host, with no scheme and no path, is not asked here.
/// - A `delete` takes the index. Deleting the last registry leaves
///   the list empty, which is written as empty and is not the default
///   three.
/// - The list is read by a provider that uses podman, one with a
///   `containers` or an `images` section, and by no other; it is
///   written here either way.
async fn registries(root: &Path, action: Registries) -> Result<(), Error> {
    match action {
        Registries::Get => {
            let config = read::<Config>(root, Which::Provider).await?;
            let mut text = Vec::new();
            for (index, host) in config.podman.registries.iter().enumerate() {
                let line = serde_json::json!({"index": index, "host": host});
                serde_json::to_writer(&mut text, &line).map_err(document::Error::Json)?;
                text.push(b'\n');
            }
            write(text).await
        }
        Registries::Add { host } => {
            provider(root, |config| add("podman.registries", &mut config.podman.registries, host)).await
        }
        Registries::Delete { index } => {
            provider(root, |config| delete("podman.registries", &mut config.podman.registries, index)).await
        }
    }
}

/// `diverge provider podman storage_path get|set|delete`: where podman
/// keeps its data, `podman.storage_path`, by these rules:
///
/// - A `get` prints the path as the provider takes it: absolute, a
///   relative one joined onto the root. One line, the path as it is
///   and not as JSON.
/// - A `set` holds the provider's lock from its first step to its
///   last — see [`hold`] — so it is refused while the provider runs,
///   and the provider cannot start while podman's data is between two
///   places.
/// - The path is written as it was given where it is relative, and
///   made absolute and plain where it is absolute. The old path and
///   the new are compared resolved; the same one is written and
///   nothing is moved.
/// - Without `--no-transfer`, podman's data is carried from the old
///   path to the new before anything is written — see
///   [`relocate`] for how, on each host. If it cannot be, nothing is
///   left moved and the path is not written: [`Error::Relocate`].
///   A provider that uses no podman — neither a `containers` section
///   nor an `images` section — keeps nothing there, and nothing is
///   carried.
/// - Then the path is written. If the write fails, the data is
///   carried back.
/// - Then the transfer is let go of: where the two paths are on
///   different filesystems the data was copied, and the old copy is
///   deleted only now. A copy that will not be deleted is
///   [`Error::Leftover`], with the data at the new path and the path
///   written.
/// - With `--no-transfer` nothing is moved or deleted. On Windows it
///   is refused while podman still holds the old path's machine: the
///   provider would refuse to start beside it.
/// - A `delete` holds the provider's lock as a `set` does, and then
///   deletes everything podman keeps at the path and everything
///   outside it that names it — see [`relocate::wipe`] — so that the
///   provider starts on the path as on one never used. The path is
///   not changed, and a provider that uses no podman has its folder
///   deleted all the same, if it has one. It is how a path is changed
///   with nothing carried on Windows: the old path's data deleted
///   first, then a `set --no-transfer`.
/// - Nothing is printed.
async fn storage_path(root: &Path, action: StoragePath) -> Result<(), Error> {
    match action {
        StoragePath::Get => {
            let config = read::<Config>(root, Which::Provider).await?;
            let path = absolute(&diverge_provider::config::resolve(root, &config.podman.storage_path))?;
            write(format!("{}\n", path.display()).into_bytes()).await
        }
        StoragePath::Set { path, no_transfer } => {
            let held = hold(root, Program::Provider).await?;
            let mut config = read::<Config>(root, Which::Provider).await?;
            let old = absolute(&diverge_provider::config::resolve(root, &config.podman.storage_path))?;
            let written = if path.is_absolute() { absolute(&path)? } else { path };
            let new = absolute(&diverge_provider::config::resolve(root, &written))?;
            let carried = if old == new || !config.podman_in_use() {
                None
            } else if no_transfer {
                #[cfg(windows)]
                relocate::machine::blocked(&old).await.map_err(Error::Relocate)?;
                None
            } else {
                Some(relocate::transfer(&old, &new).await.map_err(Error::Relocate)?)
            };
            config.podman.storage_path = written;
            let stored = match serde_yaml_ng::to_string(&config) {
                Ok(yaml) => document::write(root, Which::Provider, &yaml).await,
                Err(error) => Err(document::Error::Serialize(error)),
            };
            if let Err(error) = stored {
                if let Some(carried) = carried {
                    carried.undo().await;
                }
                return Err(error.into());
            }
            if let Some(carried) = carried {
                carried.finish().await.map_err(Error::Leftover)?;
            }
            drop(held);
            Ok(())
        }
        StoragePath::Delete => {
            let held = hold(root, Program::Provider).await?;
            let config = read::<Config>(root, Which::Provider).await?;
            let path = absolute(&diverge_provider::config::resolve(root, &config.podman.storage_path))?;
            relocate::wipe(&path).await.map_err(Error::Wipe)?;
            drop(held);
            Ok(())
        }
    }
}

/// `path` made absolute and plain, without asking the filesystem:
/// against the working directory where it is relative, its `.` and
/// separators put right.
fn absolute(path: &Path) -> Result<PathBuf, Error> {
    std::path::absolute(path).map_err(Error::Path)
}

/// A write to the provider's document: see
/// [`set`](super::setting::set).
async fn provider(root: &Path, change: impl FnOnce(&mut Config) -> Result<(), Error>) -> Result<(), Error> {
    set(root, Program::Provider, Which::Provider, change).await
}
