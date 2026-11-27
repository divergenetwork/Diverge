//! Text-to-speech.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Text-to-speech, with its arguments. The field being absent from
/// [`Toolsets`](super::Toolsets) is Hermes's own default for this
/// toolset; present is the switch thrown on.
///
/// The ElevenLabs key is OPTIONAL by the keyless-floor rule:
/// Hermes's default voice is edge-tts, which needs nothing — the
/// empty object is speech on the free voice, coherently. The key is
/// the upgrade, not the existence.
#[derive(
    Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema,
)]
pub struct Toolset {
    /// ElevenLabs as the voice: its API key, applied as
    /// `ELEVENLABS_API_KEY` in the gateway's process environment, and
    /// the harness selects the provider (`tts.provider: elevenlabs` in
    /// the config it owns). Absent is the free voice. Hermes's other
    /// keyed voices are deliberately absent at this pin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elevenlabs_api_key: Option<String>,
}
