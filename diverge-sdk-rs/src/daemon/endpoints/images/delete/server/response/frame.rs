//! What a server's response frame carries for a delete.

use serde::Serialize;
use std::fmt;

use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};
use crate::shared::error::Error;

/// A delete's answer: deleted, or why nothing was.
///
/// A delete is one question and one reply, so there is exactly one of
/// these per scope, before the finish that ends it. A payload leads
/// with one byte saying which — `0` for [`Deleted`](Self::Deleted),
/// `1` for [`InUse`](Self::InUse), `2` for
/// [`PlatformNotHeld`](Self::PlatformNotHeld), `3` for
/// [`NotFound`](Self::NotFound), `4` for
/// [`Unsupported`](Self::Unsupported), `5` for
/// [`Forbidden`](Self::Forbidden), `6` for [`Error`](Self::Error) —
/// and the rest is the error's JSON for the last, and nothing for the
/// others.
///
/// # All, or nothing
///
/// After [`Deleted`](Self::Deleted) the image is gone, or the
/// platforms named of it are. After every other answer nothing was
/// deleted: the image and every platform the daemon held of it are as
/// they were.
///
/// # Forbidden
///
/// [`Forbidden`](Self::Forbidden) is an answer every endpoint has: the
/// account the request is served for — the connection's — or the tool
/// the request came from holds no grant allowing
/// what the request asks over what it names, and nothing changed.
/// Nothing is retried. See [`grant`](crate::daemon::grant).
///
/// # As JSON
///
/// For a client that prints what the daemon answered: the variant,
/// snake case — `"deleted"`, `"in_use"`, `{"error":…}`. The wire is
/// the tag byte and the payload, never this.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Frame {
    /// The image is gone from the daemon's Store on the provider, or
    /// the platforms named of it are. Tag `0`.
    Deleted,
    /// A running container runs the image, and nothing was deleted.
    /// Tag `1`.
    InUse,
    /// A platform named is not one the daemon holds of the image
    /// there, and nothing was deleted. Tag `2`.
    PlatformNotHeld,
    /// The provider is none the daemon keeps a record of, or the
    /// daemon owns no image of the digest in its Store there. Tag `3`.
    NotFound,
    /// The provider has no Store, and there was nothing to delete.
    /// Tag `4`.
    Unsupported,
    /// The account the request is served for holds no grant allowing
    /// it: no `delete` over images reaching the provider. Tag `5`.
    Forbidden,
    /// A failure, in the daemon's words or the provider's. Tag `6`.
    ///
    /// See [`shared::error::Error`](crate::shared::error::Error) for
    /// why it says so little.
    Error(Error),
}

/// Tag for [`Frame::Deleted`].
const DELETED: u8 = 0;

/// Tag for [`Frame::InUse`].
const IN_USE: u8 = 1;

/// Tag for [`Frame::PlatformNotHeld`].
const PLATFORM_NOT_HELD: u8 = 2;

/// Tag for [`Frame::NotFound`].
const NOT_FOUND: u8 = 3;

/// Tag for [`Frame::Unsupported`].
const UNSUPPORTED: u8 = 4;

/// Tag for [`Frame::Forbidden`].
const FORBIDDEN: u8 = 5;

/// Tag for [`Frame::Error`].
const ERROR: u8 = 6;

/// A tag, then the error's JSON, or nothing.
impl Encode for Frame {
    /// The ordinary JSON failure, from the one variant that has one.
    /// The bare answers cannot fail.
    type Error = serde_json::Error;

    // Spelled out rather than `Self::Error`: this enum has a variant
    // called `Error`, so the associated type is ambiguous by that name.
    fn encode(&self, out: &mut Writer<'_>) -> Result<(), serde_json::Error> {
        let bare = |out: &mut Writer<'_>, tag: u8| {
            out.extend_from_slice(&[tag]);
            Ok(())
        };
        match self {
            Frame::Deleted => bare(out, DELETED),
            Frame::InUse => bare(out, IN_USE),
            Frame::PlatformNotHeld => bare(out, PLATFORM_NOT_HELD),
            Frame::NotFound => bare(out, NOT_FOUND),
            Frame::Unsupported => bare(out, UNSUPPORTED),
            Frame::Forbidden => bare(out, FORBIDDEN),
            Frame::Error(error) => {
                out.extend_from_slice(&[ERROR]);
                error.encode(out)
            }
        }
    }
}

impl Decode<'_> for Frame {
    /// Three ways to fail, and only one of them is a parse.
    type Error = FrameError;

    // Spelled out for the same reason as `encode` above.
    fn decode(bytes: &[u8]) -> Result<Self, FrameError> {
        let (tag, rest) = bytes.split_first().ok_or(FrameError::Empty)?;
        match *tag {
            DELETED => Ok(Frame::Deleted),
            IN_USE => Ok(Frame::InUse),
            PLATFORM_NOT_HELD => Ok(Frame::PlatformNotHeld),
            NOT_FOUND => Ok(Frame::NotFound),
            UNSUPPORTED => Ok(Frame::Unsupported),
            FORBIDDEN => Ok(Frame::Forbidden),
            ERROR => Error::decode(rest).map(Frame::Error).map_err(FrameError::Error),
            tag => Err(FrameError::UnknownTag(tag)),
        }
    }
}

/// An images delete response frame that could not be read.
#[derive(Debug)]
pub enum FrameError {
    /// No bytes at all, so not even a tag.
    Empty,
    /// A tag that is none of this frame's seven.
    UnknownTag(u8),
    /// The error did not parse.
    Error(serde_json::Error),
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameError::Empty => f.write_str("images delete response frame is empty"),
            FrameError::UnknownTag(tag) => write!(f, "unknown images delete response frame tag {tag}"),
            FrameError::Error(error) => write!(f, "images delete error did not parse: {error}"),
        }
    }
}

impl std::error::Error for FrameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FrameError::Error(error) => Some(error),
            FrameError::Empty | FrameError::UnknownTag(_) => None,
        }
    }
}
