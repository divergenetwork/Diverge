//! One volume, as a grant lists it.

use serde::{Deserialize, Serialize};

use crate::shared::Identity;

/// One volume a grant reaches: the provider that holds it, by its
/// identity, and the name that provider lists it under. A volume is a
/// provider's own and has no name anywhere else, so the pair is the
/// whole of how one is named. One JSON object,
/// `{"provider":…,"name":…}`. No other member decodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "VolumeKey")]
#[serde(deny_unknown_fields)]
pub struct Key {
    /// The provider, by its identity.
    pub provider: Identity,
    /// The name the provider lists the volume under; compared and not
    /// read.
    pub name: String,
}
