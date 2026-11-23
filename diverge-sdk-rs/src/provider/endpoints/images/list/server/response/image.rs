//! One image in the caller's Store.

use serde::{Deserialize, Serialize};

use crate::shared::containers::request::Reference;
use crate::shared::listing::Keyed;

/// An image the caller owns in its Store on the provider: one it
/// built or pushed there, by its digest.
///
/// [`digest`](Self::digest) is what it is, and the only way it is
/// named; [`bytes`](Self::bytes) how big it is; and
/// [`created`](Self::created) when the caller came to own it. That is
/// the whole of what a listing says about one; the Containerfile a
/// built one was built from is a
/// [`stat`](crate::provider::endpoints::images::stat) away, and what
/// the image says of itself — its name, its description — is in its
/// own `/.diverge/DIVERGE.md`, which a
/// [`read`](crate::provider::endpoints::images::read) answers.
///
/// # The digest is the image, and its only name
///
/// A [`Container`](crate::shared::containers::request::Container)
/// names its image by digest, and this is the digest to name: a run
/// by the identity that owns the image, naming it, runs it from the
/// Store, with no reference needed and nobody asked. A stat, a hash
/// and a delete name it by the same digest. There is no other
/// name: one image is one digest, owned once per identity.
///
/// # What changes
///
/// The bytes under a digest never change, so a listing tells an image
/// added and removed — and [`Changed`](super::Frame::Changed) when a
/// push adds platforms to what is held of it, and when a publish adds
/// a reference, and for nothing else.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Image {
    /// The manifest digest of the image as the provider holds it,
    /// `<algorithm>:<hex>`: what a container request names, and the
    /// key of the listing.
    pub digest: String,
    /// Its size in BYTES: every blob it reaches and its manifests,
    /// as the provider counts them.
    pub bytes: u64,
    /// When the caller came to own it — the build or the push ended —
    /// in SECONDS since the Unix epoch.
    pub created: u64,
    /// The platforms the image's manifest names, each a
    /// [platform string](crate::shared::platform::of), in the
    /// manifest's order: one per entry of an index, or the one an
    /// image that is not an index states in its config. What the
    /// digest is for, and never changes.
    pub manifest_platforms: Vec<String>,
    /// The platforms the caller holds of it: those of
    /// [`manifest_platforms`](Self::manifest_platforms) whose bytes
    /// are in the Store for the caller, in the same order. Equal to
    /// it, the image is whole; fewer, a
    /// [`push`](crate::provider::endpoints::images::push) adds the
    /// rest and a
    /// [`publish`](crate::provider::endpoints::images::publish) is
    /// refused until it does.
    pub platforms: Vec<String>,
    /// Where the caller has published it: each a registry and a
    /// repository path there, each once, in the order they were
    /// published. Empty until a
    /// [`publish`](crate::provider::endpoints::images::publish).
    pub references: Vec<Reference>,
}

/// Keyed by the digest: one image is one digest in one caller's
/// Store.
impl Keyed for Image {
    type Key = String;

    fn key(&self) -> String {
        self.digest.clone()
    }
}
