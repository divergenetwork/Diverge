//! The image, as the registry holds it.

use serde::{Deserialize, Serialize};

/// The image published: the registry holds it under the repository
/// named, with every tag asked for.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Published {
    /// The manifest digest the registry holds it by, `<algorithm>:<hex>`:
    /// the image's own digest, since a publish sends an image as it is
    /// held. It is `<registry>/<name>@<digest>` there from now on.
    pub digest: String,
}
