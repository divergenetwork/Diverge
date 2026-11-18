//! Images: the daemon holds none itself, and answers from the other
//! providers and its remote daemons.

use diverge_sdk::provider::client::{Manifest, OciStore};
use diverge_sdk::shared::containers::oci;
use diverge_sdk::shared::containers::request::Image;

use super::Answerer;
use crate::containers::images;

/// A provider asked whether the caller holds an image is answered
/// from the other providers connected and the remote daemons recorded:
/// an `images::read` scope is opened with the image on each, the first
/// answered held for a platform the provider runs is the run's source,
/// every other is cancelled, and its manifests, blobs and proof are
/// taken through it — see [`images`](crate::containers::images), and
/// [`images::held`]. A provider asking needs no grant, and is never
/// asked itself, having looked already. Asked for
/// the proof of possession, the daemon relays it from the source, and
/// hashes nothing itself.
impl OciStore for Answerer {
    async fn holds(&self, image: &Image) -> Option<String> {
        images::held(&self.daemon, &self.pulls, image, &self.provider).await
    }

    type Blob = images::Blob;

    async fn manifest(&self, _image: &Image, digest: &str) -> Option<Manifest> {
        images::manifest(&self.pulls, digest).await
    }

    async fn blob(&self, _image: &Image, digest: &str) -> Option<Self::Blob> {
        images::blob(&self.pulls, digest).await
    }

    async fn prove(&self, _image: &Image, request: &oci::prove::request::Request) -> Option<[u8; 32]> {
        images::prove(&self.pulls, request).await
    }
}
