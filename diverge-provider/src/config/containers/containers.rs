//! The section itself.

use serde::{Deserialize, Serialize};

/// The `containers` section: the provider runs containers, and this
/// is how much of the host they may take between them.
///
/// The section present is the `containers` capability in the
/// provider's `info`; absent, the provider runs no containers: every
/// agents run and tools run is refused `Unsupported`, before anything
/// is held, fetched or deployed. A provider that never wrote the
/// section runs none. Either field may be left out, and is then its
/// default, so `containers: {}` is a provider that runs containers on
/// the defaults.
///
/// The podman the containers run with — its registries, its storage
/// and its image cache — is the [`podman`](crate::config::podman)
/// section's, which a provider with an image Store and no containers
/// uses too.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Containers {
    /// The most the running containers may write between them, in
    /// BYTES: one cap, which two things reserve from and give back
    /// to. The rules:
    ///
    /// 1. **A container's own layer.** What a container writes over
    ///    its image: its request's `disk`, reserved for the
    ///    container's life. The image is not counted here.
    /// 2. **An ephemeral volume a container mounts.** What a
    ///    container writes into a volume in the `ephemeral` mode, kept
    ///    apart and discarded with the container: the mount's
    ///    `overlay_disk`, reserved for the container's life, one for
    ///    each mount.
    /// 3. **Refused whole.** A run whose `disk` and mounts'
    ///    `overlay_disk` together would take the sum past the cap is
    ///    refused, and nothing of it is begun.
    /// 4. **Held to what it reserved.** podman holds a container's
    ///    layer to its `disk`, and a mount's layer is of exactly its
    ///    `overlay_disk`.
    ///
    /// A FUSE serve of an ephemeral volume is not counted here: it is
    /// the volumes', counted against
    /// [`volumes.ephemeral_fuse`](crate::config::volumes::EphemeralFuse).
    pub container_overlay_disk: u64,
    /// The most memory the running containers may hold between them,
    /// in BYTES: the sum of every running container's `memory` never
    /// exceeds it, and a run that would take it over is refused. On
    /// macOS the podman machine is given exactly this much when the
    /// provider starts, since a machine there holds its memory from
    /// the host; on Windows the machine's memory is WSL's to give and
    /// is not set.
    pub memory: u64,
}

/// What a `containers` section that names nothing gives: 32 GiB of
/// overlay disk and 8 GiB of memory.
impl Default for Containers {
    fn default() -> Self {
        Containers {
            // 32 GiB.
            container_overlay_disk: 32 * 1024 * 1024 * 1024,
            // 8 GiB.
            memory: 8 * 1024 * 1024 * 1024,
        }
    }
}
