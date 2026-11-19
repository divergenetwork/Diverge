//! What this provider holds out, read off its configuration.

use diverge_sdk::provider::endpoints::info::server::response::{BuildCapabilities, Capabilities, ContainersCapabilities, ImagesCapabilities, MessagesCapabilities, StoreCapabilities, VolumeCapabilities};
use diverge_sdk::provider::endpoints::volumes::Mode;

use crate::config::Config;
use crate::host::image_manager::manifest::host;

/// The host's capabilities, each a fact of the host:
///
/// - `containers`: the `containers` section is there, which is the
///   configuration's word on whether the host runs containers at all;
///   absent, `false`, and every agents run and tools run answered
///   `Unsupported`.
/// - `images.store`: the `images` section names a Store,
///   which is the only way the host has one. Its `build` names the
///   one platform this provider builds for — a host with a Store has
///   podman, which is what builds, for the host's own platform.
/// - `containers`: the `containers` section, with the one platform
///   this provider runs: `linux` on the host's architecture.
/// - `images.registries`: every host of `podman.registries`, the
///   registries the host pulls from, anonymously,
///   on a provider that uses podman; on one that uses none — neither
///   a `containers` section nor an `images` section — empty, since
///   what an `images::read` asks of a registry's image is read out of
///   it by podman. So a provider without podman has neither a Store
///   nor a registry, and every `images::read` is answered
///   `Unsupported`.
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
///   number of either at once. An ephemeral volume is FUSE served on
///   a scratch counted against `podman.overlay_disk`, which every
///   provider has, so it is served whether the provider runs
///   containers or not; a cap of `0` refuses each serve as it
///   reserves.
/// - `messages`: the `messages` section's rate, as it is written; the
///   section absent, no object, and every listen and send answered
///   `Unsupported`.
pub fn capabilities(config: &Config) -> Capabilities {
    Capabilities {
        containers: config.containers.is_some().then(|| ContainersCapabilities { platforms: vec![host()] }),
        images: ImagesCapabilities {
            store: config.images.is_some().then(|| StoreCapabilities {
                build: Some(BuildCapabilities { platforms: vec![host()] }),
            }),
            registries: registries(config).to_vec(),
        },
        volumes_persistent: mode(config, Mode::Persistent, PERSISTENT),
        volumes_ephemeral: mode(config, Mode::Ephemeral, ANY),
        volumes_read_only: mode(config, Mode::ReadOnly, ANY),
        messages: config.messages.map(|messages| MessagesCapabilities { messages: messages.messages, seconds: messages.seconds }),
    }
}

/// The registries the provider pulls from: the `podman` section's on
/// a provider that uses podman, none on one that does not. See
/// [`capabilities`].
pub fn registries(config: &Config) -> &[String] {
    if config.podman_in_use() { &config.podman.registries } else { &[] }
}

/// The object of one volume mode, `held` being what the host does
/// with a volume of the mode once there is one. By the two lists of
/// the `volumes` section, each of which is empty where it is absent
/// and where the whole section is:
///
/// | `stores` | `fixed` | the mode's object |
/// |---|---|---|
/// | one or more | any | `held`, with `create`: a volume of any mode may be made in a store |
/// | none | one of the mode, or more | `held`, without `create`: the mode is kept, and nothing is made |
/// | none | none of the mode | every member `false`: the mode is not kept |
///
/// So `create` is the stores' alone, and the same in every mode; a
/// provider with fixed volumes and no store holds out the volumes it
/// has and makes none; and one with neither list — the section there
/// or not — keeps no volume at all.
fn mode(config: &Config, mode: Mode, held: VolumeCapabilities) -> VolumeCapabilities {
    let Some(volumes) = &config.volumes else {
        return VolumeCapabilities::default();
    };
    if !volumes.stores.is_empty() {
        return VolumeCapabilities { create: true, ..held };
    }
    if volumes.fixed.iter().any(|fixed| fixed.mode == mode) {
        return held;
    }
    VolumeCapabilities::default()
}

/// The persistent mode: mounted by any number of containers at once,
/// FUSE served once and never beside a container. `create` is not
/// said here, in this or the two after: [`mode`] says it, by the
/// stores.
const PERSISTENT: VolumeCapabilities = VolumeCapabilities {
    create: false,
    mount: true,
    fuse_serve: true,
    mount_many: true,
    fuse_serve_many: false,
    mount_and_fuse_serve: false,
};

/// A mode held by any number of containers and FUSE serves at once.
const ANY: VolumeCapabilities = VolumeCapabilities {
    create: false,
    mount: true,
    fuse_serve: true,
    mount_many: true,
    fuse_serve_many: true,
    mount_and_fuse_serve: true,
};
