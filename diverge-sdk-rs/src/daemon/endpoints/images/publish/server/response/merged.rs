//! The one image, of the parts.

use serde::{Deserialize, Serialize};

/// The one image the parts of a publish make, in the daemon's Store on
/// the provider: the provider's merge, as it answered it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct Merged {
    /// Its manifest digest, `<algorithm>:<hex>`: what is published,
    /// and the same for the same parts on every provider.
    pub digest: String,
    /// Whether this request made it: `false` where the daemon owned
    /// an image of the digest there already, and nothing was added.
    pub created: bool,
    /// Its size in BYTES as the provider counts it; absent where it
    /// was there already.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<u64>,
}
