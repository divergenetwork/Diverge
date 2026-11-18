//! Which platforms a provider runs containers for.

use serde::{Deserialize, Serialize};

use crate::shared::platform;

/// The provider runs containers, and the platforms it runs them for.
/// The object being there is the capability: a provider that runs no
/// containers sends none. The rules:
///
/// 1. **At least one platform.** `platforms` names one or more, each
///    once, each a [platform string](crate::shared::platform::of):
///    `linux/amd64`, `linux/arm64`. An object whose list is empty does
///    not decode — a provider that runs containers for no platform
///    runs none, and says so by sending no object.
/// 2. **A gate, not a choice.** A run names no platform. The provider
///    runs the image as one of the platforms listed here that the
///    image has, and which is the provider's to decide.
/// 3. **None of them is a run refused.** An image that has none of the
///    platforms listed cannot be run here.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct ContainersCapabilities {
    /// The platforms the provider runs containers for: one or more,
    /// each once.
    #[serde(deserialize_with = "platform::some")]
    pub platforms: Vec<String>,
}
