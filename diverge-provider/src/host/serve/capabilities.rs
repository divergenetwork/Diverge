//! What this provider holds out, read off its configuration.

use diverge_sdk::provider::endpoints::info::server::response::{Capabilities, MessagesCapabilities, VolumeCapabilities};
use diverge_sdk::provider::endpoints::volumes::Mode;

use crate::config::Config;

/// The host's capabilities, each a fact of the host:
///
/// - `containers`: `containers.enabled`, the configuration's word on
///   whether the host runs containers at all; `false` is every agents
///   run and tools run answered `Unsupported`.
/// - `images_store`: the `images` section names a Store,
///   which is the only way the host has one.
/// - `images_registries`: every host of `containers.podman.registries`,
///   the registries the host pulls from with the credentials it holds.
/// - The three volume modes, each kept or not by the `volumes`
///   section:
///   - a mode is kept when the section names a store — a volume of
///     any mode may be created there, however much of its capacity is
///     taken, a store with none left still kept — or a fixed volume of
///     that mode;
///   - a mode is not kept, its object every member `false`, otherwise:
///     the section absent, or naming no store and no fixed volume of
///     the mode. With no mode kept the provider keeps no volumes, and
///     every volume endpoint answers `Unsupported`.
///
///   A kept mode is held out with the most the mode allows: a
///   persistent volume mounted by any number of containers at once
///   — its image is loop-mounted once and every container binds that
///   one directory — but FUSE served once and never beside a
///   container, since a serve opens the image in userspace and the
///   image bears one writer; an ephemeral or a read-only one by any
///   number of either at once.
/// - `messages`: the `messages` section's rate, as it is written; the
///   section absent, no object, and every listen and send answered
///   `Unsupported`.
pub fn capabilities(config: &Config) -> Capabilities {
    Capabilities {
        containers: config.containers.enabled,
        images_store: config.images.is_some(),
        images_registries: config
            .containers
            .podman
            .registries
            .iter()
            .map(|registry| registry.host.clone())
            .collect(),
        volumes_persistent: if kept(config, Mode::Persistent) { PERSISTENT } else { VolumeCapabilities::default() },
        volumes_ephemeral: if kept(config, Mode::Ephemeral) { ANY } else { VolumeCapabilities::default() },
        volumes_read_only: if kept(config, Mode::ReadOnly) { ANY } else { VolumeCapabilities::default() },
        messages: config.messages.map(|messages| MessagesCapabilities { messages: messages.messages, seconds: messages.seconds }),
    }
}

/// Whether the `volumes` section keeps volumes of `mode`: a store, in
/// which a volume of any mode may be created, or a fixed volume of
/// the mode. See [`capabilities`].
fn kept(config: &Config, mode: Mode) -> bool {
    let Some(volumes) = &config.volumes else {
        return false;
    };
    let stores = volumes.stores.as_ref().is_some_and(|stores| !stores.is_empty());
    let fixed = volumes.fixed.iter().flatten().any(|fixed| fixed.mode == mode);
    stores || fixed
}

/// The persistent mode: mounted by any number of containers at once,
/// FUSE served once and never beside a container.
const PERSISTENT: VolumeCapabilities = VolumeCapabilities {
    mount: true,
    fuse_serve: true,
    mount_many: true,
    fuse_serve_many: false,
    mount_and_fuse_serve: false,
};

/// A mode held by any number of containers and FUSE serves at once.
const ANY: VolumeCapabilities = VolumeCapabilities {
    mount: true,
    fuse_serve: true,
    mount_many: true,
    fuse_serve_many: true,
    mount_and_fuse_serve: true,
};
