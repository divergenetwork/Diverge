//! What a server's response frame carries for an image merge.

use std::fmt;

use super::Merged;
use crate::shared::error::Error;
use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};

/// The image merged; the word that the caller owns it already; a
/// refusal; or the news that the provider could not.
///
/// One of these on channel `0`, then the scope finishes. A payload
/// leads with one byte saying which — `0` for [`Merged`](Self::Merged),
/// `1` for [`Exists`](Self::Exists), `2` for
/// [`NotFound`](Self::NotFound), `3` for
/// [`PlatformNotHeld`](Self::PlatformNotHeld), `4` for
/// [`PlatformRepeated`](Self::PlatformRepeated), `5` for
/// [`InsufficientCapacity`](Self::InsufficientCapacity), `6` for
/// [`Unsupported`](Self::Unsupported), `7` for [`Error`](Self::Error)
/// — and the rest is that variant's own bytes; the bare refusals
/// carry nothing.
///
/// | the scope ends with | means |
/// |----------------------|-------|
/// | [`Merged`](Self::Merged), then a finish | the new image is in the caller's Store, and a listing will show it |
/// | [`Exists`](Self::Exists), then a finish | the caller owned that digest already; nothing was added |
/// | [`NotFound`](Self::NotFound), then a finish | the caller owns no image by one of the digests, and nothing was made |
/// | [`PlatformNotHeld`](Self::PlatformNotHeld), then a finish | a source does not hold a platform taken from it, and nothing was made |
/// | [`PlatformRepeated`](Self::PlatformRepeated), then a finish | a platform was named more than once, and nothing was made |
/// | [`InsufficientCapacity`](Self::InsufficientCapacity), then a finish | the Store has no room for it, and nothing was made |
/// | [`Unsupported`](Self::Unsupported), then a finish | the provider has no Store |
/// | an [`Error`](Self::Error), then a finish | it does not exist, for some other reason |
///
/// # The refusals are answers, not errors
///
/// Each is something a caller acts on: push the platform that is not
/// held, name each platform once, delete an image and ask again. An
/// [`Error`](Self::Error) is none of them.
#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    /// The new image is in the caller's Store. Tag `0`.
    Merged(Merged),
    /// The merge made a digest the caller already owns, and nothing
    /// was added: the digest. Tag `1`.
    Exists(String),
    /// The caller owns no image by one of the digests named, and
    /// nothing was made. Tag `2`.
    NotFound,
    /// A source does not hold a platform the request takes from it,
    /// and nothing was made. Tag `3`.
    ///
    /// Carries nothing. A caller pushes the platform into the source
    /// and asks again, or takes it from another.
    PlatformNotHeld,
    /// A platform is named more than once across the request, and
    /// nothing was made. Tag `4`.
    PlatformRepeated,
    /// The Store has no room for the image, and nothing was made. Tag
    /// `5`.
    InsufficientCapacity,
    /// The provider has no Store — its `images.store` is absent, see
    /// [`info`](crate::provider::endpoints::info) — and nothing was
    /// made. Tag `6`.
    ///
    /// Carries nothing. A caller asks another provider; nothing about
    /// the request is the cause.
    Unsupported,
    /// A failure. Tag `7`.
    ///
    /// See [`shared::error::Error`](crate::shared::error::Error) for
    /// why it says so little.
    Error(Error),
}

/// Tag for [`Frame::Merged`].
const MERGED: u8 = 0;

/// Tag for [`Frame::Exists`].
const EXISTS: u8 = 1;

/// Tag for [`Frame::NotFound`].
const NOT_FOUND: u8 = 2;

/// Tag for [`Frame::PlatformNotHeld`].
const PLATFORM_NOT_HELD: u8 = 3;

/// Tag for [`Frame::PlatformRepeated`].
const PLATFORM_REPEATED: u8 = 4;

/// Tag for [`Frame::InsufficientCapacity`].
const INSUFFICIENT_CAPACITY: u8 = 5;

/// Tag for [`Frame::Unsupported`].
const UNSUPPORTED: u8 = 6;

/// Tag for [`Frame::Error`].
const ERROR: u8 = 7;

/// Two formats, and the tag chooses between them: the image merged
/// and the digest owned already are postcard, as the rest of the
/// Store is, and the error is JSON.
impl Encode for Frame {
    /// One failure per half, and they are different libraries'.
    type Error = FrameEncodeError;

    // Spelled out rather than `Self::Error`: this enum has a variant
    // called `Error`, so the associated type is ambiguous by that name.
    fn encode(&self, out: &mut Writer<'_>) -> Result<(), FrameEncodeError> {
        match self {
            Frame::Merged(merged) => {
                out.extend_from_slice(&[MERGED]);
                postcard::to_io(merged, &mut *out).map(|_| ()).map_err(FrameEncodeError::Merged)
            }
            Frame::Exists(digest) => {
                out.extend_from_slice(&[EXISTS]);
                postcard::to_io(digest, &mut *out).map(|_| ()).map_err(FrameEncodeError::Merged)
            }
            Frame::NotFound => {
                out.extend_from_slice(&[NOT_FOUND]);
                Ok(())
            }
            Frame::PlatformNotHeld => {
                out.extend_from_slice(&[PLATFORM_NOT_HELD]);
                Ok(())
            }
            Frame::PlatformRepeated => {
                out.extend_from_slice(&[PLATFORM_REPEATED]);
                Ok(())
            }
            Frame::InsufficientCapacity => {
                out.extend_from_slice(&[INSUFFICIENT_CAPACITY]);
                Ok(())
            }
            Frame::Unsupported => {
                out.extend_from_slice(&[UNSUPPORTED]);
                Ok(())
            }
            Frame::Error(error) => {
                out.extend_from_slice(&[ERROR]);
                error.encode(out).map_err(FrameEncodeError::Error)
            }
        }
    }
}

impl Decode<'_> for Frame {
    /// Four ways to fail, and each names which half failed.
    type Error = FrameDecodeError;

    // Spelled out for the same reason as `encode` above.
    fn decode(bytes: &[u8]) -> Result<Self, FrameDecodeError> {
        let (tag, rest) = bytes.split_first().ok_or(FrameDecodeError::Empty)?;
        match *tag {
            MERGED => postcard::from_bytes(rest).map(Frame::Merged).map_err(FrameDecodeError::Merged),
            EXISTS => postcard::from_bytes(rest).map(Frame::Exists).map_err(FrameDecodeError::Merged),
            NOT_FOUND => Ok(Frame::NotFound),
            PLATFORM_NOT_HELD => Ok(Frame::PlatformNotHeld),
            PLATFORM_REPEATED => Ok(Frame::PlatformRepeated),
            INSUFFICIENT_CAPACITY => Ok(Frame::InsufficientCapacity),
            UNSUPPORTED => Ok(Frame::Unsupported),
            ERROR => Error::decode(rest).map(Frame::Error).map_err(FrameDecodeError::Error),
            tag => Err(FrameDecodeError::UnknownTag(tag)),
        }
    }
}

/// An image merge response that could not be written.
#[derive(Debug)]
pub enum FrameEncodeError {
    /// The image merged, or the digest owned already, did not
    /// serialize.
    Merged(postcard::Error),
    /// The error did not serialize.
    Error(serde_json::Error),
}

impl fmt::Display for FrameEncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameEncodeError::Merged(error) => write!(f, "image merged did not serialize: {error}"),
            FrameEncodeError::Error(error) => write!(f, "image merge error did not serialize: {error}"),
        }
    }
}

impl std::error::Error for FrameEncodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FrameEncodeError::Merged(error) => Some(error),
            FrameEncodeError::Error(error) => Some(error),
        }
    }
}

/// An image merge response that could not be read.
#[derive(Debug)]
pub enum FrameDecodeError {
    /// No bytes at all, so not even a tag.
    Empty,
    /// A tag that is none of this frame's eight.
    UnknownTag(u8),
    /// The image merged, or the digest owned already, did not parse.
    Merged(postcard::Error),
    /// The error did not parse.
    Error(serde_json::Error),
}

impl fmt::Display for FrameDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameDecodeError::Empty => f.write_str("image merge response frame is empty"),
            FrameDecodeError::UnknownTag(tag) => write!(f, "unknown image merge response frame tag {tag}"),
            FrameDecodeError::Merged(error) => write!(f, "image merged did not parse: {error}"),
            FrameDecodeError::Error(error) => write!(f, "image merge error did not parse: {error}"),
        }
    }
}

impl std::error::Error for FrameDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FrameDecodeError::Merged(error) => Some(error),
            FrameDecodeError::Error(error) => Some(error),
            FrameDecodeError::Empty | FrameDecodeError::UnknownTag(_) => None,
        }
    }
}
