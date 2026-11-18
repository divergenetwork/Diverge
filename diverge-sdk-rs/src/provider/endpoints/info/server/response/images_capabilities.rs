//! What a provider does with images.

use serde::{Deserialize, Serialize};

use crate::shared::platform;

/// What a provider does with images: whether it keeps a Store, what
/// its Store does, and the registries it pulls from. A member absent
/// is that part absent, so a provider that does nothing with images
/// sends no object, or an empty one.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct ImagesCapabilities {
    /// The provider keeps the Store for its callers, and what its
    /// Store does: see [`StoreCapabilities`]. With the object there —
    /// whatever its members — the provider answers
    /// [`images::list`](crate::provider::endpoints::images::list),
    /// [`stat`](crate::provider::endpoints::images::stat),
    /// [`push`](crate::provider::endpoints::images::push),
    /// [`edit`](crate::provider::endpoints::images::edit) and
    /// [`delete`](crate::provider::endpoints::images::delete), and
    /// [`images::read`](crate::provider::endpoints::images::read) looks
    /// in it first. Absent — no object — each of the five answers
    /// `Unsupported`, as [`images::build`](crate::provider::endpoints::images::build)
    /// does, and `read` answers `Unsupported` when
    /// [`registries`](ImagesCapabilities::registries) is empty as
    /// well.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store: Option<StoreCapabilities>,
    /// The registries the provider pulls from: each a host as a
    /// [`Reference`](crate::shared::containers::request::Reference)
    /// names it — `docker.io`, `ghcr.io`, `registry.example.com:5000`
    /// — and the whole of them. An
    /// [`images::read`](crate::provider::endpoints::images::read)
    /// consults exactly these after the caller's Store, and a reference
    /// naming any other is ignored; a run's image whose references
    /// name none of these, and that neither the caller's Store nor the
    /// caller holds, cannot be supplied. Absent — empty — the provider
    /// pulls from no registry, and `read` answers `Unsupported` when
    /// [`store`](ImagesCapabilities::store) is absent as well.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub registries: Vec<String>,
}

impl ImagesCapabilities {
    /// Whether the provider builds: it keeps a Store, and its Store
    /// builds.
    pub fn builds(&self) -> bool {
        self.store.as_ref().is_some_and(|store| store.build.is_some())
    }
}

/// What a provider's Store does beyond being kept. The Store itself —
/// listing it, pushing into it, merging and publishing what is in it,
/// editing and deleting it — is this object being there at all.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct StoreCapabilities {
    /// The provider builds into the Store, and for which platforms: it
    /// answers
    /// [`images::build`](crate::provider::endpoints::images::build).
    /// See [`BuildCapabilities`]. Absent — no object — a build answers
    /// `Unsupported`, the first and only response of its scope, and
    /// nothing is built; every other endpoint of the Store is served
    /// as it is with the object there.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<BuildCapabilities>,
}

/// The provider builds, and the platforms it builds for. The object
/// being there is the capability: a provider that does not build
/// sends none. The rules:
///
/// 1. **At least one platform.** `platforms` names one or more, each
///    once, each a [platform string](crate::shared::platform::of). An
///    object whose list is empty does not decode.
/// 2. **A build names its platforms**, and each must be one listed
///    here: a build naming any other is answered
///    [`PlatformUnsupported`](crate::provider::endpoints::images::build::server::response::Frame::PlatformUnsupported),
///    and nothing is built.
/// 3. **Building is not running.** The platforms built for need not be
///    the platforms
///    [`containers`](super::Capabilities::containers) runs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct BuildCapabilities {
    /// The platforms the provider builds for: one or more, each once.
    #[serde(deserialize_with = "platform::some")]
    pub platforms: Vec<String>,
}
