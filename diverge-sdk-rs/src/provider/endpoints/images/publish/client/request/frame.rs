//! What a client's request frame carries for an image publish.

use serde::{Deserialize, Serialize};

use super::Credential;
use crate::shared::containers::request::Reference;
use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};

/// Publish an image the caller holds to a registry.
///
/// # The rules
///
/// 1. **The image is the caller's.** [`digest`](Self::digest) is one
///    the caller owns in its Store; any other is
///    [`NotFound`](crate::provider::endpoints::images::publish::server::response::Frame::NotFound).
/// 2. **Whole, or not at all.** A publish sends every platform the
///    image's manifest names, and names none itself. An image that
///    holds only some of them — its
///    [`platforms`](crate::provider::endpoints::images::list::server::response::Image::platforms)
///    fewer than its
///    [`manifest_platforms`](crate::provider::endpoints::images::list::server::response::Image::manifest_platforms)
///    — is
///    [`Incomplete`](crate::provider::endpoints::images::publish::server::response::Frame::Incomplete):
///    a registry takes no manifest that names what it was not given.
/// 3. **A registry the provider lists.** The
///    [`reference`](Self::reference)'s registry is one of the
///    provider's `images.registries`, the ones it pulls from; any
///    other is
///    [`RegistryUnlisted`](crate::provider::endpoints::images::publish::server::response::Frame::RegistryUnlisted).
/// 4. **As it is held.** The image is sent byte for byte, so the
///    registry holds it by the digest the Store does.
/// 5. **The tags, or none.** Each of [`tags`](Self::tags) is put on
///    the image there. With none, the image is published by its digest
///    alone: `<registry>/<name>@<digest>`.
/// 6. **Recorded.** A publish that ends
///    [`Published`](crate::provider::endpoints::images::publish::server::response::Frame::Published)
///    adds the reference to the image's
///    [`references`](crate::provider::endpoints::images::list::server::response::Image::references),
///    unless it is there already.
///
/// # On the wire
///
/// Postcard after the tag, as the rest of the Store's requests are.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Frame {
    /// Which image, by its manifest digest, `<algorithm>:<hex>`.
    pub digest: String,
    /// Where it goes: the registry, and the repository path there.
    /// See [`Reference`].
    pub reference: Reference,
    /// The tags to put on it there, each a tag as a registry takes
    /// one: one to 128 of letters, digits, `_`, `.` and `-`, the first
    /// not `.` or `-`. Empty, the image is published by digest alone.
    pub tags: Vec<String>,
    /// What to write as: see [`Credential`]. `None` asks the registry
    /// as nobody, which a registry that takes writes from nobody
    /// allows. One postcard byte, `0` for none and `1` followed by the
    /// credential.
    pub credential: Option<Credential>,
}

/// This frame's tag among the scope-opening requests.
///
/// One byte at the front of the payload, which is what tells a reader
/// which request it holds. See the table in
/// [`endpoints`](crate::provider::endpoints) for the whole allocation.
const TAG: u8 = 19;

impl Encode for Frame {
    /// Postcard's own failure. The tag cannot fail.
    type Error = postcard::Error;

    fn encode(&self, out: &mut Writer<'_>) -> Result<(), Self::Error> {
        out.extend_from_slice(&[TAG]);
        postcard::to_io(self, &mut *out)?;
        Ok(())
    }
}

impl Decode<'_> for Frame {
    /// Three ways to fail, and only one of them is postcard's.
    type Error = FrameError;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        let (tag, rest) = bytes.split_first().ok_or(FrameError::Empty)?;
        if *tag != TAG {
            return Err(FrameError::UnexpectedTag(*tag));
        }
        postcard::from_bytes(rest).map_err(FrameError::Body)
    }
}

/// An image publish request that could not be read.
#[derive(Debug)]
pub enum FrameError {
    /// No bytes at all, so not even a tag.
    Empty,
    /// A tag naming some other request.
    UnexpectedTag(u8),
    /// The request did not parse.
    Body(postcard::Error),
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameError::Empty => f.write_str("image publish request frame is empty"),
            FrameError::UnexpectedTag(tag) => {
                write!(f, "expected image publish request tag {TAG}, found {tag}")
            }
            FrameError::Body(error) => write!(f, "image publish request did not parse: {error}"),
        }
    }
}

impl std::error::Error for FrameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FrameError::Body(error) => Some(error),
            FrameError::Empty | FrameError::UnexpectedTag(_) => None,
        }
    }
}
