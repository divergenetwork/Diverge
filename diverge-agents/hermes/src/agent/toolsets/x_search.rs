//! X (Twitter) search.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// X (Twitter) search over the xAI API. The field being absent from
/// [`Toolsets`](super::Toolsets) is the toolset off; present is the
/// switch thrown on.
///
/// The key path ONLY, deliberately: Hermes's own dispatch prefers
/// the key, and the xAI subscription OAuth bearer answers this tool
/// in a degraded no-citation mode — the refreshing login is a worse
/// product here, not a missing feature.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct Toolset {
    /// The xAI API key, applied as `XAI_API_KEY` in the gateway's
    /// process environment. The xai provider is applied under the
    /// same variable: an agent that names both names one key, and
    /// two different keys are the registration's refusal.
    pub api_key: String,
}
