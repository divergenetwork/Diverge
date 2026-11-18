//! A blob the registry took.

use serde::{Deserialize, Serialize};

/// One blob the provider sent to the registry: the publish's progress.
/// The provider's
/// [`Blob`](crate::provider::endpoints::images::publish::server::response::Blob),
/// as it gave it.
///
/// Sent once per blob the registry took, after it accepted the blob
/// whole, and never for a blob the registry held already.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct Uploaded {
    /// The blob's digest, `<algorithm>:<hex>`.
    pub digest: String,
    /// Its size in BYTES.
    pub bytes: u64,
}
