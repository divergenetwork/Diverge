//! One volume made visible inside a container.

use serde::{Deserialize, Serialize};

use crate::provider::endpoints::volumes::Mode;

/// A volume from a listing, mounted into the container.
///
/// Host side first, then the container side — source before
/// destination, the order a mount reads in everywhere else.
///
/// # Why the host side is a name and an offset
///
/// Because a host path is not something a caller is allowed to state.
/// [`volume_name`](Self::volume_name) is a
/// [`Volume::name`](crate::provider::endpoints::volumes::list::server::response::Volume::name)
/// the provider published, and
/// [`volume_relative_path`](Self::volume_relative_path) descends from
/// wherever that maps to — so a caller reaches a subdirectory of
/// something it was offered, and nothing else.
///
/// That is the same access model as every
/// [`volumes`](crate::provider::endpoints::volumes) endpoint, and it holds for
/// the same reason: a provider never validates a path, it resolves a name it
/// chose and then descends. A caller cannot escape upward, because
/// there is no component it can write that means "up" — the offset is
/// components, and `..` is a name, not an instruction.
///
/// # The mode is the volume's, and the mount states it
///
/// Whether what a container writes into the volume is in the volume
/// when the container ends, discarded with the container, or refused
/// inside it is the volume's [`Mode`], as its
/// [listing](crate::provider::endpoints::volumes::list::server::response::Volume::mode)
/// reports it and as a [`create`](crate::provider::endpoints::volumes::create)
/// stated or an [`edit`](crate::provider::endpoints::volumes::edit) changed it
/// — a fact of the volume, the same for every container that mounts
/// it. A mount does not change it: it STATES the mode it means the
/// volume to have, [`mode`](Self::mode), and a volume in another mode
/// refuses the whole run,
/// [`VolumeMode`](crate::shared::containers::response::VolumeMode),
/// before anything is fetched or deployed, so that a container never
/// runs on a volume behaving otherwise than its caller meant. A
/// request naming one volume at several paths names one mode for it.
/// How many may mount a volume is the volume's too, as far as the
/// provider's [`info`](crate::provider::endpoints::info) capability
/// for the mode goes: up to any number of the caller's containers at
/// once, whatever the mode, each of which may name it at several
/// paths — a persistent volume being one filesystem they share, an
/// ephemeral one a volume each starts from afresh. A second holder
/// the provider does not allow is
/// refused with
/// [`VolumeMounted`](crate::shared::containers::response::VolumeMounted)
/// or
/// [`VolumeFuseServed`](crate::shared::containers::response::VolumeFuseServed),
/// a mode the provider does not mount with
/// [`VolumeMountUnsupported`](crate::shared::containers::response::VolumeMountUnsupported).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct VolumeMount {
    /// Which offered volume, by the name a listing gave it.
    ///
    /// Names come from
    /// [`Volume::name`](crate::provider::endpoints::volumes::list::server::response::Volume::name)
    /// and mean nothing outside the provider that published them.
    pub volume_name: String,
    /// How far into that volume to start, as path components
    /// relative to it.
    ///
    /// Empty mounts the volume itself, which is the common case;
    /// anything else mounts a subdirectory of it.
    pub volume_relative_path: Vec<String>,
    /// The mode the mount means the volume to have: `persistent`,
    /// `ephemeral` or `read_only`. A volume in another mode refuses
    /// the run; nothing changes a volume's mode but an edit.
    pub mode: Mode,
    /// The most bytes the mount's own layer may hold, for an ephemeral
    /// volume. The rules:
    ///
    /// 1. **Present exactly when [`mode`](Self::mode) is `ephemeral`**,
    ///    and absent otherwise: only an ephemeral volume has a layer
    ///    of its own, which holds what the container writes into the
    ///    volume and is discarded with the container. A mount that has
    ///    it or lacks it the other way round refuses the run.
    /// 2. **Reserved.** The provider sets the bytes aside for the
    ///    container's life, out of the capacity the run's `disk` is
    ///    set aside from — what its
    ///    [`capacity`](crate::provider::endpoints::capacity) answers
    ///    as `container_disk` — and refuses the run when it cannot, as
    ///    it refuses one whose `disk` it cannot set aside. A FUSE
    ///    serve's `overlay_disk` is set aside from another capacity,
    ///    `fuse_disk`, and takes no room from this one.
    /// 3. **Held to it.** A write inside the container that would take
    ///    the layer past the bytes fails there, as a write to a full
    ///    disk does, and the container goes on. `0` is a layer nothing
    ///    can be written into.
    /// 4. **One layer for each mount.** A volume named at several
    ///    paths is a layer, and a number, for each.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overlay_disk: Option<u64>,
    /// Where it appears inside the container, as path components from
    /// the container's root.
    ///
    /// Empty means the root itself, which a provider will almost
    /// certainly refuse — the image's own filesystem is there.
    pub container_path: Vec<String>,
}
