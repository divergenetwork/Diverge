//! Qwen's portal.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Qwen's portal, over OAuth. Single-use rotating refresh tokens,
/// so the credential REFRESHES: the arguments give the login once,
/// the run rotates it, and the program keeps the rotated state in
/// its own database instead of silently burning the login.
///
/// APPLICATION: unlike the other OAuth providers this state is not
/// Hermes's own — it is the Qwen CLI's token file. The harness
/// writes the login to `~/.qwen/oauth_creds.json` at the REAL
/// home directory (Hermes hardcodes that path; it ignores
/// `$HERMES_HOME`) before Hermes starts. On token refresh Hermes
/// rewrites the creds file; after the gateway exits the file is read
/// back as the run left it and kept for the next run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Provider {
    /// The discriminator. Always `qwen-oauth`.
    pub provider: QwenOauth,
    /// The login, as the arguments give it: the OAuth state document,
    /// a JSON object, exactly as the Qwen CLI's token file holds it. Used until the
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
pub enum QwenOauth {
    #[default]
    QwenOauth,
}
