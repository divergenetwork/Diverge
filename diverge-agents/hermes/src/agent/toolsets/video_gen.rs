//! Video generation.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Video generation over FAL — the same deliberate one-provider
/// scope as image generation. The field being absent from
/// [`Toolsets`](super::Toolsets) is the toolset off; present is the
/// switch thrown on, and the harness pins `video_gen.provider: fal`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct Toolset {
    /// The FAL key, applied as `FAL_KEY` in the gateway's process
    /// environment. Image generation is applied under the same
    /// variable: an agent that names both names one key, and two
    /// different keys are the registration's refusal.
    pub fal_key: String,
}
