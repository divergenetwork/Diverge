//! What a server's response frame carries for an images merge.

use serde::Serialize;
use std::fmt;

use super::{Added, Blob, Merged};
use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};
use crate::shared::error::Error;

/// What happened, as it happened, or the one answer that ends the
/// merge.
///
/// A merge is a stream with an end: zero or more
/// [`Blob`](Self::Blob) and [`Added`](Self::Added), as the images are
/// brought onto the provider, then exactly one of the others, then
/// the finish. [`Merged`](Self::Merged) is the merge done; every
/// other is why it was not, and what landed before it stays in the
/// Store.
///
/// A payload leads with one byte saying which, and the rest is that
/// variant's own JSON; a bare answer carries nothing after its tag.
///
/// # As JSON
///
/// For a client that prints what the daemon answered: the variant,
/// snake case, over its payload, and a bare variant the string alone.
/// The wire is the tag byte and the payload, never this.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Frame {
    /// A blob was pulled onto the provider. Tag `0`.
    Blob(Blob),
    /// A set of platforms landed in the Store there. Tag `1`.
    Added(Added),
    /// The image is in the Store: the merge's end. Tag `2`.
    Merged(Merged),
    /// The caller holds no `merge` grant over images reaching the
    /// provider. Tag `3`.
    Forbidden,
    /// The daemon keeps no record of the provider named. Tag `4`.
    NotFound,
    /// The provider named keeps no Store. Tag `5`.
    Unsupported,
    /// An image named, or a platform taken of it, is one no source
    /// supplies. Tag `6`.
    Unavailable,
    /// Two images take the same platform, and nothing was asked of
    /// the provider. Tag `7`.
    PlatformRepeated,
    /// A platform an image names is not one it has. Tag `8`.
    PlatformNotFound,
    /// The provider's Store has no room. Tag `9`.
    InsufficientCapacity,
    /// A source supplied an image and did not prove it holds it. Tag
    /// `10`.
    ProofFailed,
    /// An image is not one the provider takes. Tag `11`.
    Malformed,
    /// A failure, in the daemon's words or the provider's. Tag `12`.
    Error(Error),
}

/// Tag for [`Frame::Blob`].
const BLOB: u8 = 0;

/// Tag for [`Frame::Added`].
const ADDED: u8 = 1;

/// Tag for [`Frame::Merged`].
const MERGED: u8 = 2;

/// Tag for [`Frame::Forbidden`].
const FORBIDDEN: u8 = 3;

/// Tag for [`Frame::NotFound`].
const NOT_FOUND: u8 = 4;

/// Tag for [`Frame::Unsupported`].
const UNSUPPORTED: u8 = 5;

/// Tag for [`Frame::Unavailable`].
const UNAVAILABLE: u8 = 6;

/// Tag for [`Frame::PlatformRepeated`].
const PLATFORM_REPEATED: u8 = 7;

/// Tag for [`Frame::PlatformNotFound`].
const PLATFORM_NOT_FOUND: u8 = 8;

/// Tag for [`Frame::InsufficientCapacity`].
const INSUFFICIENT_CAPACITY: u8 = 9;

/// Tag for [`Frame::ProofFailed`].
const PROOF_FAILED: u8 = 10;

/// Tag for [`Frame::Malformed`].
const MALFORMED: u8 = 11;

/// Tag for [`Frame::Error`].
const ERROR: u8 = 12;

/// The tag, then the variant's own JSON; a bare answer carries
/// nothing after its tag.
impl Encode for Frame {
    /// The ordinary JSON failure. The tag cannot fail.
    type Error = serde_json::Error;

    // Spelled out rather than `Self::Error`: this enum has a variant
    // called `Error`, so the associated type is ambiguous by that name.
    fn encode(&self, out: &mut Writer<'_>) -> Result<(), serde_json::Error> {
        let bare = |out: &mut Writer<'_>, tag: u8| {
            out.extend_from_slice(&[tag]);
            Ok(())
        };
        match self {
            Frame::Blob(value) => {
                out.extend_from_slice(&[BLOB]);
                serde_json::to_writer(out, value)
            }
            Frame::Added(value) => {
                out.extend_from_slice(&[ADDED]);
                serde_json::to_writer(out, value)
            }
            Frame::Merged(value) => {
                out.extend_from_slice(&[MERGED]);
                serde_json::to_writer(out, value)
            }
            Frame::Forbidden => bare(out, FORBIDDEN),
            Frame::NotFound => bare(out, NOT_FOUND),
            Frame::Unsupported => bare(out, UNSUPPORTED),
            Frame::Unavailable => bare(out, UNAVAILABLE),
            Frame::PlatformRepeated => bare(out, PLATFORM_REPEATED),
            Frame::PlatformNotFound => bare(out, PLATFORM_NOT_FOUND),
            Frame::InsufficientCapacity => bare(out, INSUFFICIENT_CAPACITY),
            Frame::ProofFailed => bare(out, PROOF_FAILED),
            Frame::Malformed => bare(out, MALFORMED),
            Frame::Error(error) => {
                out.extend_from_slice(&[ERROR]);
                error.encode(out)
            }
        }
    }
}

impl Decode<'_> for Frame {
    /// Four ways to fail, and two of them are a parse.
    type Error = FrameError;

    // Spelled out for the same reason as `encode` above.
    fn decode(bytes: &[u8]) -> Result<Self, FrameError> {
        let (tag, rest) = bytes.split_first().ok_or(FrameError::Empty)?;
        match *tag {
            BLOB => serde_json::from_slice(rest).map(Frame::Blob).map_err(FrameError::Body),
            ADDED => serde_json::from_slice(rest).map(Frame::Added).map_err(FrameError::Body),
            MERGED => serde_json::from_slice(rest).map(Frame::Merged).map_err(FrameError::Body),
            FORBIDDEN => Ok(Frame::Forbidden),
            NOT_FOUND => Ok(Frame::NotFound),
            UNSUPPORTED => Ok(Frame::Unsupported),
            UNAVAILABLE => Ok(Frame::Unavailable),
            PLATFORM_REPEATED => Ok(Frame::PlatformRepeated),
            PLATFORM_NOT_FOUND => Ok(Frame::PlatformNotFound),
            INSUFFICIENT_CAPACITY => Ok(Frame::InsufficientCapacity),
            PROOF_FAILED => Ok(Frame::ProofFailed),
            MALFORMED => Ok(Frame::Malformed),
            ERROR => Error::decode(rest).map(Frame::Error).map_err(FrameError::Error),
            tag => Err(FrameError::UnknownTag(tag)),
        }
    }
}

/// A response frame of an images merge that could not be read.
#[derive(Debug)]
pub enum FrameError {
    /// No bytes at all, so not even a tag.
    Empty,
    /// A tag that is none of this frame's thirteen.
    UnknownTag(u8),
    /// What the tag promised did not parse.
    Body(serde_json::Error),
    /// The error did not parse.
    Error(serde_json::Error),
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameError::Empty => f.write_str("the response frame of an images merge is empty"),
            FrameError::UnknownTag(tag) => write!(f, "unknown response frame tag {tag} of an images merge"),
            FrameError::Body(error) => write!(f, "an answer of an images merge did not parse: {error}"),
            FrameError::Error(error) => write!(f, "an error of an images merge did not parse: {error}"),
        }
    }
}

impl std::error::Error for FrameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FrameError::Body(error) | FrameError::Error(error) => Some(error),
            FrameError::Empty | FrameError::UnknownTag(_) => None,
        }
    }
}
