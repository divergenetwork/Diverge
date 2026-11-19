//! The section itself.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{EphemeralFuse, Fixed, Store};

/// The `volumes` section: where new volumes may be created, and which
/// volumes exist already. Two lists, either of which may be empty; a
/// section with both empty keeps no volume, as no section does.
/// A member the document leaves out is as [`Volumes::default`] has
/// it: the lists empty, and `ephemeral_fuse` on, at its defaults.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Volumes {
    /// Places the provider may create a volume in, in order of
    /// preference. Absent is empty, and an empty list is not written.
    /// With one store or more a volume of any mode may be created:
    /// every mode is held out, each with `create`. With none no
    /// client can create a volume: the modes the fixed volumes are in
    /// are held out without `create`, a create is answered
    /// `ModeUnsupported` and `volumes::create_capacity` answers `0`;
    /// and with no fixed volume either the provider keeps no volume
    /// mode, and every volume endpoint answers `Unsupported`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stores: Vec<Store>,
    /// Volumes that exist before any client asks, offered in a
    /// listing to every identity, or to the ones each names;
    /// each is listed as `provided`, and is neither edited nor
    /// deleted by a client. Absent is empty, and an empty list is not
    /// written.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixed: Vec<Fixed>,
    /// Where a FUSE serve of an ephemeral volume keeps what is
    /// written through it, and how much: see [`EphemeralFuse`]. The
    /// rules:
    ///
    /// 1. **Left out, on.** A `volumes` section that does not say is
    ///    one with [`EphemeralFuse::default`]: `provider/ephemeral_fuse`
    ///    under the root, and 32 GiB.
    /// 2. **`null`, off.** The member written as `null` is the
    ///    provider serving no ephemeral volume over FUSE.
    /// 3. **Always written.** The member is written as it is, `null`
    ///    included, so that off stays off when the document is read
    ///    again.
    pub ephemeral_fuse: Option<EphemeralFuse>,
}

/// No store, no fixed volume, and ephemeral FUSE serving on at its
/// defaults: what a `volumes` section is where it says nothing.
impl Default for Volumes {
    fn default() -> Self {
        Volumes {
            stores: Vec::new(),
            fixed: Vec::new(),
            ephemeral_fuse: Some(EphemeralFuse::default()),
        }
    }
}

impl Volumes {
    /// The host directories the volumes live in: every store's path
    /// and every fixed volume's, in the configuration's order. What a
    /// podman machine must be able to see, on the host that has one.
    pub fn paths(&self) -> Vec<PathBuf> {
        self.stores
            .iter()
            .map(|store| store.path.clone())
            .chain(self.fixed.iter().map(|fixed| fixed.path.clone()))
            .collect()
    }
}
