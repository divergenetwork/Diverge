//! Where an ephemeral volume's FUSE serve keeps what is written
//! through it, and how much.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The `ephemeral_fuse` of the `volumes` section: the provider serves
/// volumes in the `ephemeral` mode over FUSE, and this is where each
/// serve's own layer is kept and how much the layers may hold between
/// them. The rules:
///
/// 1. **Present, the capability.** With the member there, the
///    provider holds out `fuse_serve`, `fuse_serve_many` and
///    `mount_and_fuse_serve` for the `ephemeral` mode, where it keeps
///    volumes of the mode at all. A `volumes` section that leaves the
///    member out has it, at its defaults: `provider/ephemeral_fuse`
///    under the root, and 32 GiB. Written as `null`, it holds out
///    none of the three: a FUSE serve of an ephemeral volume is answered
///    `Unsupported`, no directory is made, and nothing is reserved.
///    Whether the provider runs containers has no say in it.
/// 2. **A layer for each serve.** What is written through a FUSE
///    serve of an ephemeral volume is kept apart from the volume, in
///    a file of the serve's own under [`path`](Self::path), and
///    discarded when the serve ends.
/// 3. **Reserved.** A serve's `overlay_disk` is taken from
///    [`capacity`](Self::capacity) when it opens and given back when
///    it ends. One that would take the sum past the capacity is
///    refused, and nothing opens.
/// 4. **Held to what it reserved.** A write through a serve that
///    would take its layer past its `overlay_disk` is refused whole,
///    and the serve goes on.
/// 5. **Its own.** Neither a container's `disk` nor an ephemeral
///    volume a container mounts is counted here: those are the
///    `containers` section's `container_overlay_disk`, kept where
///    podman keeps its data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EphemeralFuse {
    /// The directory the layers are kept in, a file each, made by the
    /// provider at its start if it is not there and emptied then of
    /// what an earlier life left; a relative path is resolved against
    /// the root, the directory that holds `provider.yaml`. No podman
    /// machine is shown it: a FUSE serve is the provider's own, on the
    /// host.
    pub path: PathBuf,
    /// The most the layers may hold between them, in BYTES: the sum of
    /// every open serve's `overlay_disk` never exceeds it. `0` is
    /// refused at the provider's start.
    pub capacity: u64,
}

/// `provider/ephemeral_fuse` under the root, and 32 GiB: what a
/// `volumes` section has where it does not say.
impl Default for EphemeralFuse {
    fn default() -> Self {
        EphemeralFuse {
            path: PathBuf::from("provider/ephemeral_fuse"),
            // 32 GiB.
            capacity: 32 * 1024 * 1024 * 1024,
        }
    }
}
