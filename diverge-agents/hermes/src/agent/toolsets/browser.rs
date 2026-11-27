//! Browser automation.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Browser automation, with its arguments. The field being absent
/// from [`Toolsets`](super::Toolsets) is Hermes's own default for
/// this toolset; present is the switch thrown on.
///
/// The one slot is optional because the default backend is the
/// container's own headless Chromium, which needs nothing — the
/// slot points the toolset at a browser somewhere else instead,
/// and the empty object is the local browser, coherently.
#[derive(
    Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema,
)]
pub struct Toolset {
    /// The remote backend, one of [`Remote`]'s; absent = the local
    /// headless Chromium.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote: Option<Remote>,
}

/// One remote browser backend. Untagged — Browserbase is an object
/// carrying its key and its project, a raw endpoint is an object
/// carrying its address — and the two share no member, so no marker
/// is needed. One slot for both because they are the same role:
/// Browserbase IS a CDP provider, and a request naming a raw
/// endpoint and a managed one would be contradicting itself.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Remote {
    /// Browserbase. The pair is one credential, applied in the
    /// gateway's process environment, and the harness pins
    /// `browser.cloud_provider: browserbase`.
    Browserbase {
        /// The API key, applied as `BROWSERBASE_API_KEY`.
        api_key: String,
        /// The project's id, applied as `BROWSERBASE_PROJECT_ID`.
        project_id: String,
    },
    /// Any browser reachable over CDP, applied as
    /// `BROWSER_CDP_URL`. No credential.
    Cdp {
        /// The endpoint.
        cdp_url: String,
    },
}
