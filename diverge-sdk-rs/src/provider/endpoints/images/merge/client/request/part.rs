//! One source of a merge.

use serde::{Deserialize, Serialize};

use crate::shared::platform;

/// One image a merge takes platforms from, and which.
///
/// The image is one the caller holds, by its digest; the platforms
/// are among the ones it holds of it — its
/// [`platforms`](crate::provider::endpoints::images::list::server::response::Image::platforms),
/// not merely the ones its manifest names. A source may hold
/// platforms the merge does not take, and two sources may hold the
/// same platform: what is taken from each is what is named here.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Part {
    /// Which image, by its manifest digest, `<algorithm>:<hex>`.
    pub digest: String,
    /// The platforms to take from it: one or more, each once, each a
    /// [platform string](crate::shared::platform::of). A list that is
    /// empty does not decode.
    #[serde(deserialize_with = "platform::some")]
    pub platforms: Vec<String>,
}
