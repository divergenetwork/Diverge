//! The image, as the registry holds it.

use serde::{Deserialize, Serialize};

use crate::daemon::agent_log::Identity;
use crate::shared::containers::request::Reference;

/// The image published: the registry holds it under the repository
/// named, with every tag asked for.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct Published {
    /// The provider it was done on, and in whose Store of the daemon's
    /// the image is: the one the request named, or the one the daemon
    /// picked.
    pub provider: Identity,
    /// The manifest digest the registry holds it by,
    /// `<algorithm>:<hex>`: a part's own digest where one part was
    /// published as it is, the merged image's otherwise. It is
    /// `<registry>/<name>@<digest>` there from now on.
    pub digest: String,
    /// Whether this request made the image in the daemon's Store on
    /// the provider. `true`: the daemon owned no image of the digest
    /// there before, and it was brought in, or merged, by this
    /// request, and is kept as the request's description. `false`: it
    /// was there already, is kept as it was, and the request's
    /// description was not used.
    pub created: bool,
    /// Where it went, as the request named it.
    pub reference: Reference,
    /// The tags put on it there, as the request named them.
    pub tags: Vec<String>,
}
