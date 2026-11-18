//! One image a publish takes platforms from.

use serde::{Deserialize, Deserializer, Serialize};

use crate::shared::platform;

/// One image of a publish, by its digest, and the platforms taken of
/// it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct Part {
    /// Which image, by its manifest digest, `<algorithm>:<hex>`.
    pub digest: String,
    /// The platforms to take of it: one or more, each once, each a
    /// [platform string](crate::shared::platform::of) such as
    /// `linux/amd64`. Absent, every platform its manifest names. A
    /// list that is empty does not parse.
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "platforms")]
    pub platforms: Option<Vec<String>>,
}

/// The platforms taken, where the member is present: one or more, as
/// [`platform::some`] reads them.
fn platforms<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Vec<String>>, D::Error> {
    platform::some(deserializer).map(Some)
}
