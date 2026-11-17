//! One setting of the provider's document, read or written by itself:
//! its port, its identity, whom it accepts, whom it dials, the
//! registries it pulls from, where podman keeps its data, how much
//! its image cache may hold, whether it runs containers, and the
//! volumes it provides. Every one keeps the rules of
//! [`setting`](super::setting).

use std::path::{Path, PathBuf};

use diverge_provider::config::Config;
use diverge_provider::config::containers;
use diverge_provider::config::outgoing;
use diverge_provider::config::volumes as volumes_section;
use diverge_provider::host::relocate;
use diverge_sdk::shared;

use super::Error;
use super::setting::{add, delete, hold, identities, line, number, read, set, write};
use crate::arguments::{Bytes, Containers, Fixed, Identity, ImageCacheDisk, Incoming, Outgoing, Podman, Port, Registries, StoragePath, Volumes};
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

/// `diverge provider containers …`: whether the provider runs
/// containers, and the settings of that. The `containers` section of
/// the document is the provider running containers, and the section
/// absent is it running none; so, by these rules:
///
/// - `enabled` prints `true` where the section is there and `false`
///   where it is not. It is answered whether the provider is running
///   or not.
/// - `enable` makes the section, with its defaults, where it is
///   absent. Where it is there, nothing is changed and nothing is
///   written: its settings are kept, never put back to their
///   defaults.
/// - `disable` removes the section. Its settings go with it, since
///   they are the section's and the document has nowhere else to
///   keep them: an `enable` after is the defaults again.
/// - A setting's `get` prints the number, or `null` where the section
///   is absent: the default is not printed, since nothing is in
///   effect.
/// - A setting's `set` is refused where the section is absent,
///   [`Error::Disabled`]: it never enables containers by the way,
///   since a provider that runs containers needs podman and the
///   container proxy at its next start.
/// - `enable`, `disable` and a `set` each hold the provider's lock
///   and are refused while it runs, as every write is.
pub async fn containers(root: &Path, action: Containers) -> Result<(), Error> {
    match action {
        Containers::Enabled => line(&read::<Config>(root, Which::Provider).await?.containers.is_some()).await,
        Containers::Enable => {
            provider(root, |config| {
                config.containers.get_or_insert_with(containers::Containers::default);
                Ok(())
            })
            .await
        }
        Containers::Disable => {
            provider(root, |config| {
                config.containers = None;
                Ok(())
            })
            .await
        }
        Containers::ContainerOverlayDisk(action) => {
            bytes(root, action, |section| &mut section.container_overlay_disk).await
        }
        Containers::Memory(action) => bytes(root, action, |section| &mut section.memory).await,
    }
}

/// One setting of the `containers` section that is a number of
/// bytes, read or written: `of` is the setting in the section. See
/// [`containers`] for the rules.
async fn bytes(root: &Path, action: Bytes, of: impl FnOnce(&mut containers::Containers) -> &mut u64) -> Result<(), Error> {
    match action {
        Bytes::Get => {
            let mut config = read::<Config>(root, Which::Provider).await?;
            line(&config.containers.as_mut().map(|section| *of(section))).await
        }
        Bytes::Set { value } => {
            provider(root, |config| match &mut config.containers {
                Some(section) => {
                    *of(section) = value;
                    Ok(())
                }
                None => Err(Error::Disabled { section: "containers", program: Program::Provider }),
            })
            .await
        }
    }
}

/// `diverge provider volumes …`: the `volumes` section, there or
/// not, and the fixed volumes in it. The section holds two lists —
/// the stores, and the fixed volumes — and the provider keeps
/// volumes when either holds something: a section with both empty
/// keeps none, as no section does. So, by these rules:
///
/// - `enabled` prints `true` where the section is there and `false`
///   where it is not. It says the section is there, and not that the
///   provider keeps volumes.
/// - `enable` makes the section, with both lists empty, where it is
///   absent; where it is there nothing is changed or written.
/// - `disable` removes the section, and both lists with it.
/// - `fixed get` prints each fixed volume as one JSON line with its
///   `index` first, in the document's order. An absent section is an
///   empty list: nothing is printed, and it is no error.
/// - `fixed add` puts the volume after the last. A name listed
///   already is refused, [`Error::Listed`]: a volume is asked for by
///   its name, and two of one name are one that cannot be reached.
///   The path is written as given, and nothing on disk is made or
///   looked at: the provider finds a path that is no directory at its
///   start.
/// - `fixed delete` takes the index, and removes the entry: the
///   directory and what is in it are left as they are.
/// - `fixed add` and `fixed delete` are refused where the section is
///   absent, [`Error::Disabled`]: neither makes the section by the
///   way.
/// - `enable`, `disable`, `fixed add` and `fixed delete` each hold
///   the provider's lock and are refused while it runs.
///
/// The stores have no command here.
pub async fn volumes(root: &Path, action: Volumes) -> Result<(), Error> {
    let absent = || Error::Disabled { section: "volumes", program: Program::Provider };
    match action {
        Volumes::Enabled => line(&read::<Config>(root, Which::Provider).await?.volumes.is_some()).await,
        Volumes::Enable => {
            provider(root, |config| {
                config.volumes.get_or_insert_with(volumes_section::Volumes::default);
                Ok(())
            })
            .await
        }
        Volumes::Disable => {
            provider(root, |config| {
                config.volumes = None;
                Ok(())
            })
            .await
        }
        Volumes::Fixed(Fixed::Get) => {
            let config = read::<Config>(root, Which::Provider).await?;
            let mut text = Vec::new();
            for (index, fixed) in config.volumes.iter().flat_map(|section| &section.fixed).enumerate() {
                let mut line = serde_json::json!({
                    "index": index,
                    "name": fixed.name,
                    "path": fixed.path,
                    "bytes": fixed.bytes,
                    "mode": fixed.mode,
                });
                if let Some(hook) = &fixed.authorize_hook {
                    line["authorize_hook"] = serde_json::Value::String(hook.clone());
                }
                serde_json::to_writer(&mut text, &line).map_err(document::Error::Json)?;
                text.push(b'\n');
            }
            write(text).await
        }
        Volumes::Fixed(Fixed::Add { name, path, bytes, mode, authorize_hook }) => {
            provider(root, |config| {
                let section = config.volumes.as_mut().ok_or_else(absent)?;
                if let Some(index) = section.fixed.iter().position(|fixed| fixed.name == name) {
                    return Err(Error::Listed { list: "volumes.fixed", item: format!("a volume named `{name}`"), index });
                }
                section.fixed.push(volumes_section::Fixed { name, path, bytes, mode, authorize_hook });
                Ok(())
            })
            .await
        }
        Volumes::Fixed(Fixed::Delete { index }) => {
            provider(root, |config| {
                let section = config.volumes.as_mut().ok_or_else(absent)?;
                delete("volumes.fixed", &mut section.fixed, index)
            })
            .await
        }
    }
}

/// `diverge provider podman …`: the settings of the `podman` section,
/// each named as the section names it.
pub async fn podman(root: &Path, action: Podman) -> Result<(), Error> {
    match action {
        Podman::Registries(action) => registries(root, action).await,
        Podman::StoragePath(action) => storage_path(root, action).await,
        Podman::ImageCacheDisk(action) => image_cache_disk(root, action).await,
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

/// `diverge provider podman image_cache_disk get|set`: the most
/// podman's image cache may hold, `podman.image_cache_disk`, in
/// bytes. A `get` prints the number the provider runs with, the
/// default where the document has none. A `set` writes it as given:
/// nothing in the cache is removed here, and a cache already over a
/// smaller number is trimmed by the provider at its next deploy, as
/// it trims at every one. The number is read by a provider that uses
/// podman, and written here either way.
async fn image_cache_disk(root: &Path, action: ImageCacheDisk) -> Result<(), Error> {
    match action {
        ImageCacheDisk::Get => number(read::<Config>(root, Which::Provider).await?.podman.image_cache_disk).await,
        ImageCacheDisk::Set { value } => {
            provider(root, |config| {
                config.podman.image_cache_disk = value;
                Ok(())
            })
            .await
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
