//! The tool as a patch sees it.

use serde::{Deserialize, Serialize};

use crate::daemon::endpoints::agents::create::client::request::{FuseMount, VolumeMount};

/// The members of a tool an edit may change, as one JSON document:
/// what a patch is applied to, and what its result must be again —
/// every member present, a member the patch takes away read as
/// absent or empty, and no member this type does not have.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Editable {
    /// The name, unique across the daemon's standalone and connected
    /// tools together; another's is the edit's `InUse`. `null`, the tool has no name, and is reached
    /// once and for all only.
    #[serde(default)]
    pub name: Option<String>,
    /// Volumes of the provider the create pinned the tool to, as it
    /// is to mount them: see [`VolumeMount`]. Ordered, and applied in
    /// order; no mount's path, in any list, is a prefix of another's.
    /// A tool pinned to no provider has none, and a patch that gives
    /// it one is the edit's error.
    #[serde(default)]
    pub volume_mounts: Vec<VolumeMount>,
    /// Files of providers' volumes served live across the daemon,
    /// mounted one each over FUSE: see [`FuseMount`].
    #[serde(default)]
    pub fuse_file_mounts: Vec<FuseMount>,
    /// Directories of providers' volumes served live across the
    /// daemon, mounted one each over FUSE: see [`FuseMount`].
    #[serde(default)]
    pub fuse_directory_mounts: Vec<FuseMount>,
}
