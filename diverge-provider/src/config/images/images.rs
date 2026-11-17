//! The Store: the images clients own on the provider.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Where the Store is kept — the images clients build or push on the
/// provider, each owned by digest — and how many bytes it may take.
///
/// A directory and a number, as a volume store is. Under the
/// directory the provider keeps every blob ONCE, content-addressed,
/// however many identities own images reaching it; one ownership
/// record per identity and image, under the identity's folders named
/// as the volume store names them; and cache records of images a
/// caller supplied to a run and proved, which nobody owns — see
/// [`Store`](crate::host::image_manager::Store). The provider's own
/// registry serves the blobs to a run, one random token per pull.
/// Nothing owned is evicted: an image stays until every identity
/// owning it deletes it, and a build or a push the Store has no room
/// for is refused with the answer. Cache records are trimmed, oldest
/// first, whenever room is needed.
///
/// Either field may be left out, and is then its default, so that
/// `images: {}` is a provider keeping a Store under
/// `provider/images` that may hold 32 GiB. The section absent is the
/// provider keeping no Store at all: the defaults are the section's,
/// and nothing of them holds where there is no section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Images {
    /// The directory the Store lives under, made by the provider at
    /// its start if it is not there; a relative path is resolved
    /// against the root, the directory that holds `provider.yaml`. On
    /// macOS the podman machine is made seeing it, since a build's
    /// export is written through podman. Absent, `provider/images`:
    /// beside podman's data, under the root.
    pub path: PathBuf,
    /// The most the Store may hold, in BYTES: every blob, each counted
    /// once. A push is refused before any blob moves when the sizes
    /// its manifest declares would pass it; a build that leaves the
    /// Store over it is undone; either is answered insufficient
    /// capacity. `0` is refused at the provider's start. Absent,
    /// 32 GiB, which is what podman's image cache and the containers'
    /// overlays each default to.
    pub capacity: u64,
}

/// What an `images` section that names nothing gives: the Store under
/// `provider/images` beside `provider.yaml`, holding 32 GiB at most.
impl Default for Images {
    fn default() -> Self {
        Images {
            path: PathBuf::from("provider/images"),
            // 32 GiB.
            capacity: 32 * 1024 * 1024 * 1024,
        }
    }
}
