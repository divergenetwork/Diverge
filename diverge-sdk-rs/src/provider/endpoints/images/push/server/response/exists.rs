//! The image, owned already.

use serde::{Deserialize, Serialize};

/// The caller already owns the digest in its Store: nothing was
/// pushed, nothing moved, and the image the caller has is the one it
/// named.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Exists {
    /// The manifest digest, `<algorithm>:<hex>`, as the request named
    /// it.
    pub digest: String,
}
