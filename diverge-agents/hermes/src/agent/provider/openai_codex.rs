//! The ChatGPT Codex backend.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The ChatGPT Codex backend, over external OAuth. Single-use
/// rotating refresh tokens, so the credential REFRESHES: the
/// arguments give the login once, the run rotates it, and the
/// program keeps the rotated state in its own database instead of
/// silently burning the login.
///
/// APPLICATION: the harness writes the login as the
/// `providers.openai-codex` entry of `$HERMES_HOME/auth.json` before
/// Hermes starts — provider selection rides the config the harness
/// already owns, never the file's `active_provider`. On token
/// refresh Hermes rewrites the entry in place; after the gateway
/// exits the entry is read back as the run left it and kept for the
/// next run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Provider {
    /// The discriminator. Always `openai-codex`.
    pub provider: OpenaiCodex,
    /// The login, as the arguments give it: the OAuth state document,
    /// a JSON object, exactly as Hermes keeps the entry. Used until the
    /// program holds a refreshed one kept for this same value: see
    /// [`credentials`](crate::credentials).
    pub oauth: serde_json::Map<String, serde_json::Value>,
}

/// [`Provider`]'s discriminator.
///
/// One variant, and the reason the [union](super::Provider) can be
/// untagged: no other provider's arguments can produce this value.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum OpenaiCodex {
    #[default]
    OpenaiCodex,
}
