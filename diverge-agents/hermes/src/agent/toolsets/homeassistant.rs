//! Home Assistant control.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Home Assistant control, with its arguments. The field being
/// absent from [`Toolsets`](super::Toolsets) is Hermes's own
/// default for this toolset; present is the switch thrown on.
///
/// The url is REQUIRED here even though Hermes treats it as
/// optional: its fallback is `homeassistant.local`, an mDNS name
/// that is dead inside a container — an instance the caller does
/// not name is an instance this run cannot reach.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct Toolset {
    /// The instance, applied as `HASS_URL` in the gateway's
    /// process environment.
    pub url: String,
    /// The long-lived access token, static by Home Assistant's own
    /// design and never rotated, applied as `HASS_TOKEN` in the
    /// gateway's process environment.
    pub token: String,
}
