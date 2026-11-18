//! What a client's request frame carries for an image merge.

use serde::{Deserialize, Serialize};

use super::Part;
use crate::shared::description::Description;
use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};

/// Make one image of the platforms of several the caller holds.
///
/// # What is made
///
/// A new image: an index whose entries are exactly the platforms
/// named, each the platform's own image as its source holds it. It is
/// the caller's, by its digest, and its sources are as they were —
/// a merge changes nothing that exists. No layer is copied: the new
/// image reaches the blobs its sources reach, each kept once.
///
/// # The rules
///
/// 1. **Each source is the caller's.** A digest the caller owns no
///    image by is
///    [`NotFound`](crate::provider::endpoints::images::merge::server::response::Frame::NotFound).
/// 2. **Each platform taken is held.** A platform a [`Part`] names
///    that its image does not hold is
///    [`PlatformNotHeld`](crate::provider::endpoints::images::merge::server::response::Frame::PlatformNotHeld):
///    a manifest naming a platform is not the platform being there.
/// 3. **Each platform once.** A platform named by two parts, or twice
///    in one, is
///    [`PlatformRepeated`](crate::provider::endpoints::images::merge::server::response::Frame::PlatformRepeated):
///    an image has one image per platform. The sources themselves may
///    overlap; naming the platforms part by part is how one is chosen.
/// 4. **One form.** The index is written the same way whatever the
///    order of the request — its entries ordered by platform — so the
///    same platforms of the same images are the same digest on every
///    provider. A merge that makes a digest the caller owns already
///    adds nothing, and answers
///    [`Exists`](crate::provider::endpoints::images::merge::server::response::Frame::Exists).
///
/// # On the wire
///
/// Postcard after the tag, as the rest of the Store's requests are.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Frame {
    /// The sources, and what is taken from each. One or more.
    pub images: Vec<Part>,
    /// What the caller says of the new image, at most 132 characters,
    /// or `None` for nothing. One postcard byte, `0` for none and `1`
    /// followed by the text.
    pub description: Option<Description>,
}

/// This frame's tag among the scope-opening requests.
///
/// One byte at the front of the payload, which is what tells a reader
/// which request it holds. See the table in
/// [`endpoints`](crate::provider::endpoints) for the whole allocation.
const TAG: u8 = 18;

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

/// An image merge request that could not be read.
#[derive(Debug)]
pub enum FrameError {
    /// No bytes at all, so not even a tag.
    Empty,
    /// A tag naming some other request.
    UnexpectedTag(u8),
    /// The request did not parse: a source with no platform among it.
    Body(postcard::Error),
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameError::Empty => f.write_str("image merge request frame is empty"),
            FrameError::UnexpectedTag(tag) => {
                write!(f, "expected image merge request tag {TAG}, found {tag}")
            }
            FrameError::Body(error) => write!(f, "image merge request did not parse: {error}"),
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
