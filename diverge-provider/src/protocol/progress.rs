//! What a build or a push says as it goes, and how it ends.

use diverge_sdk::provider::endpoints::images::build::server::response::{Built, Exists};
use diverge_sdk::provider::endpoints::images::merge::server::response::Merged;
use diverge_sdk::provider::endpoints::images::publish::server::response::{Blob as Uploaded, Published};
use diverge_sdk::provider::endpoints::images::push::server::response::{Blob, Exists as Present, Pushed};

/// One item of a build, as an
/// [`ImageManager::Build`](super::image_manager::ImageManager::Build)
/// yields them: a line the build tool wrote, as it wrote it; or, last,
/// the image built, the word that the caller owns what was built
/// already, or the word that the Store has no room for it. A build
/// that failed is the stream's error, after whatever lines it said,
/// and nothing is owned. The handler sends each as the endpoint's
/// frame and finishes the scope at the first that ends the build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Progress {
    /// A line of the build's output, decoded: a caller shows it to a
    /// person and does not parse it.
    Output(String),
    /// The caller owns the image now, under its digest. Nothing
    /// follows.
    Built(Built),
    /// The build produced a digest the caller owns already: nothing
    /// changed. Nothing follows.
    Exists(Exists),
    /// The Store has no room for the image. Nothing is owned, and
    /// nothing follows.
    InsufficientCapacity,
    /// A platform named is not one the provider builds for. Nothing
    /// is built, and nothing follows.
    PlatformUnsupported,
}

/// One item of a push, as an
/// [`ImageManager::Push`](super::image_manager::ImageManager::Push)
/// yields them: a blob taken from the caller, as it lands; then
/// exactly one of the ends. A push that failed otherwise is the
/// stream's error, and nothing is owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pushing {
    /// One blob taken from the caller, verified and kept.
    Blob(Blob),
    /// The caller owns the image now. Nothing follows.
    Pushed(Pushed),
    /// The caller owned the digest already. Nothing follows.
    Exists(Present),
    /// The Store has no room for the image, by the sizes its manifest
    /// declares; no blob moved. Nothing follows.
    InsufficientCapacity,
    /// No registry the provider uses serves the image, and the caller
    /// does not supply it. Nothing follows.
    Unavailable,
    /// The caller did not prove that it holds the image. What the
    /// push brought in is removed. Nothing follows.
    ProofFailed,
    /// A manifest the provider does not take, bytes that are not
    /// their digest's, or a blob of another size than declared.
    /// Nothing follows.
    Malformed,
    /// A platform named is not one the image has. Nothing moved, and
    /// nothing follows.
    PlatformNotFound,
}

/// How a merge ended, as
/// [`ImageManager::merge`](super::image_manager::ImageManager::merge)
/// answers it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Merging {
    /// The caller owns the new image now.
    Merged(Merged),
    /// The merge made a digest the caller owns already: the digest.
    Exists(String),
    /// A source is not the caller's.
    NotFound,
    /// A source does not hold a platform taken from it.
    PlatformNotHeld,
    /// A platform is named more than once.
    PlatformRepeated,
    /// The Store has no room for the image.
    InsufficientCapacity,
}

/// One item of a publish, as an
/// [`ImageManager::Publish`](super::image_manager::ImageManager::Publish)
/// yields them: a blob the registry took, as it lands; then exactly
/// one of the ends. A publish that failed otherwise is the stream's
/// error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Publishing {
    /// One blob the registry took from the provider.
    Blob(Uploaded),
    /// The registry holds the image, with every tag. Nothing follows.
    Published(Published),
    /// The caller owns no image by the digest. Nothing follows.
    NotFound,
    /// The image does not hold every platform its manifest names.
    /// Nothing follows.
    Incomplete,
    /// The registry is none of the provider's. Nothing follows.
    RegistryUnlisted,
    /// The repository path or a tag is not one. Nothing follows.
    Malformed,
    /// The registry did not take the credential. Nothing follows.
    Unauthorized,
    /// The credential may not write to the repository. Nothing
    /// follows.
    Denied,
}
