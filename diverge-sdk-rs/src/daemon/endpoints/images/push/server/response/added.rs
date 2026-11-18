//! Platforms of the image, added.

use serde::{Deserialize, Serialize};

use crate::shared::containers::response::Source;

/// Some of the platforms a push named, now in the daemon's Store on
/// the provider: one of the pushes a push is made of, ended well.
///
/// The rules:
///
/// 1. **One per source.** A push whose source is the daemon is one
///    push to the provider for each source the daemon found, each
///    naming the platforms that source supplies, all at once. Each
///    that ends with the platforms in the Store is one of these, sent
///    as it ends. A push whose source is the client is one push, and
///    sends one.
/// 2. **All of these, or none of them.** The platforms of one are
///    added together: a push to the provider that fails adds none of
///    its own, and takes back none of another's.
/// 3. **What was added stays.** A push that ends in a failure after
///    one of these still holds what it reported.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct Added {
    /// The platforms now held, each a
    /// [platform string](crate::shared::platform::of).
    pub platforms: Vec<String>,
    /// The image's size in BYTES as the provider counts it once these
    /// are held; absent where the provider held them all already.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<u64>,
    /// Where the provider took them from: its own Store, where it held
    /// them already; a registry it uses; or its caller, the daemon,
    /// which proved the source holds them.
    pub source: Source,
}
