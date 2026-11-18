//! What a client's request frame carries for an image deletion.

use serde::{Deserialize, Serialize};

use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};

/// Destroy an image in the caller's Store, or some of its platforms.
///
/// # The rules
///
/// 1. **No platform named, the whole image.** With
///    [`platforms`](Self::platforms) empty the caller's ownership of
///    the image goes, as the sections below state.
/// 2. **Named, those platforms and no other.** The caller's record of
///    the image holds the platforms named no longer. The digest is
///    the image still: its manifest names every platform it did, and
///    a listing reports fewer
///    [`platforms`](crate::provider::endpoints::images::list::server::response::Image::platforms)
///    than
///    [`manifest_platforms`](crate::provider::endpoints::images::list::server::response::Image::manifest_platforms),
///    as after a push of some of them.
/// 3. **All of them, or none.** A platform named that the caller does
///    not hold of the image is
///    [`PlatformNotHeld`](crate::provider::endpoints::images::delete::server::response::Frame::PlatformNotHeld),
///    and nothing is removed. A platform named twice does not decode.
/// 4. **Every platform held, named, is the whole image.** Naming all
///    the caller holds of the image is rule 1: an image held for no
///    platform is no image.
/// 5. **Nothing shared is lost.** What goes is the caller's record of
///    the platforms. A blob another platform of the image, another
///    image, another owner or the provider's cache still reaches
///    stays.
///
/// # A digest, and the same access model as everything else
///
/// The one field is an
/// [`Image::digest`](crate::provider::endpoints::images::list::server::response::Image::digest)
/// from a listing. A caller cannot delete an image it does not own,
/// and cannot probe for what exists by deleting and reading the
/// error — the provider resolves a digest against the caller's own
/// Store, and nothing else. What the caller loses is its ownership:
/// a blob another owner's image or the provider's cache still reaches
/// stays, and is not the caller's to count.
///
/// # It is not undoable and there is no confirmation
///
/// One frame destroys the image. Confirmation belongs where a person
/// is, which is above this. A caller that wants the image again
/// builds or pushes it again.
///
/// # An image in use is refused
///
/// An image a running container of the caller runs is never deleted,
/// and neither is a platform of it the provider runs containers for:
/// a provider answers
/// [`InUse`](crate::provider::endpoints::images::delete::server::response::Frame::InUse)
/// and changes nothing. A caller that wants it gone stops the
/// container first and asks again.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Frame {
    /// Which image, by its manifest digest, `<algorithm>:<hex>`.
    pub digest: String,
    /// The platforms to delete of it: each once, each a
    /// [platform string](crate::shared::platform::of). Empty, the
    /// whole image.
    #[serde(deserialize_with = "distinct")]
    pub platforms: Vec<String>,
}

/// Zero or more platforms, each a platform string and each once: a
/// string that is none, or one named twice, does not decode.
fn distinct<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Vec<String>, D::Error> {
    let platforms = Vec::<String>::deserialize(deserializer)?;
    for (index, platform) in platforms.iter().enumerate() {
        if !crate::shared::platform::valid(platform) {
            return Err(serde::de::Error::custom("a platform is `<os>/<architecture>` or `<os>/<architecture>/<variant>`"));
        }
        if platforms[..index].contains(platform) {
            return Err(serde::de::Error::custom("a platform is named once"));
        }
    }
    Ok(platforms)
}

/// This frame's tag among the scope-opening requests.
///
/// One byte at the front of the payload, which is what tells a reader
/// which request it holds. The frame layer does not discriminate them
/// — [`ClientFrame::Request`](crate::wire::frame::client::ClientFrame::Request)
/// is one type carrying bytes — so the distinction has to be in the
/// bytes, and each request owns the value that names it.
///
/// See the table in [`endpoints`](crate::provider::endpoints) for the whole
/// allocation. The values are chosen across modules that do not know
/// about each other, so the table is the only place they can be seen
/// at once.
const TAG: u8 = 21;

/// Postcard, matching [`volumes`](crate::provider::endpoints::volumes) and the
/// rest of this family's store.
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

/// An image deletion request that could not be read.
#[derive(Debug)]
pub enum FrameError {
    /// No bytes at all, so not even a tag.
    Empty,
    /// A tag naming some other request.
    ///
    /// A reader that dispatched on the tag will not see this. One that
    /// assumed which request it held, and was wrong, will — which is
    /// the point of checking a tag rather than skipping it.
    UnexpectedTag(u8),
    /// The request did not parse.
    Body(postcard::Error),
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameError::Empty => f.write_str("image deletion request frame is empty"),
            FrameError::UnexpectedTag(tag) => {
                write!(f, "expected image deletion request tag {TAG}, found {tag}")
            }
            FrameError::Body(error) => write!(f, "image deletion request did not parse: {error}"),
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
