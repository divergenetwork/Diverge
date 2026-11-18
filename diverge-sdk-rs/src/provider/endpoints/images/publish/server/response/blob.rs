//! A blob uploaded to the registry.

use serde::{Deserialize, Serialize};

/// One blob the provider uploaded to the registry: the publish's
/// progress.
///
/// Sent once per blob the registry took from the provider, after the
/// registry accepted it whole, and never for a blob the registry held
/// already. A publish that uploads nothing sends none.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Blob {
    /// The blob's digest, `<algorithm>:<hex>`.
    pub digest: String,
    /// Its size in BYTES.
    pub bytes: u64,
}
