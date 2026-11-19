//! The section itself.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{EphemeralFuse, Fixed, Store};

/// The `volumes` section: where new volumes may be created, and which
/// volumes exist already. Two lists, either of which may be empty; a
/// section with both empty keeps no volume, as no section does.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
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
    /// written through it, and how much: see [`EphemeralFuse`].
    /// Absent, the provider serves no ephemeral volume over FUSE, and
    /// the member is not written.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ephemeral_fuse: Option<EphemeralFuse>,
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
