//! Image generation.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Image generation over FAL, the built-in path (Hermes's other
/// image providers — openai, krea, deepinfra, xai — are deliberately
/// absent at this pin). The field being absent from
/// [`Toolsets`](super::Toolsets) is the toolset off; present is the
/// switch thrown on, and the harness pins `image_gen.provider: fal`.
/// Generation has no keyless path, so the switch and the key are one
/// ask.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct Toolset {
    /// The FAL key, applied as `FAL_KEY` in the gateway's process
    /// environment. Video generation is applied under the same
    /// variable: an agent that names both names one key, and two
    /// different keys are the registration's refusal.
    pub fal_key: String,
}
