//! The image a merge made.

use serde::{Deserialize, Serialize};

/// The image a merge made, now in the caller's Store.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Merged {
    /// The manifest digest of the new image, `<algorithm>:<hex>`: the
    /// digest of the index the provider wrote, and what every later
    /// request names it by.
    pub digest: String,
    /// Its size in BYTES: every blob it reaches and its manifests, as
    /// the provider counts them. The blobs are its sources' own.
    pub bytes: u64,
}
