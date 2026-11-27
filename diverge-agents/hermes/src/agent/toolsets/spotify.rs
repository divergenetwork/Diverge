//! Spotify control.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Spotify control, with its arguments. Its login REFRESHES: Hermes
/// refreshes the PKCE state on effectively every Spotify tool call
/// and rewrites the store each time, so the arguments give the login
/// once and the program keeps its rotated form in its own database,
/// for the next run. The field being absent from
/// [`Toolsets`](super::Toolsets) is Hermes's own default for this
/// toolset; present is the switch thrown on.
///
/// APPLICATION: the harness sets `HERMES_SPOTIFY_CLIENT_ID` to
/// [`client_id`](Self::client_id) in the gateway's process
/// environment, and writes the login as the
/// `providers.spotify` entry of `$HERMES_HOME/auth.json` (Hermes
/// keeps this entry inert for provider selection by its own
/// design); after the gateway exits the entry is read back as the
/// run left it and kept for the next run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Toolset {
    /// The caller's own Spotify app (PKCE — there is no client
    /// secret, and Hermes ships no default app). Static and public.
    pub client_id: String,
    /// The login, as the arguments give it: the login document, a
    /// JSON object, exactly as Hermes keeps the entry. Used until the
    /// program holds a refreshed one kept for this same value: see
    /// [`credentials`](crate::credentials).
    pub oauth: serde_json::Map<String, serde_json::Value>,
}
