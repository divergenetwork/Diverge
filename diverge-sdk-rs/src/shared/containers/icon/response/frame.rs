//! What an icon channel answers.

use std::fmt;

use crate::shared::error::Error;
use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};

/// One of an image's icons, that the image ships none, why it could
/// not be read, or that it is too large to be.
///
/// One frame is all there is — this is not a stream, and a channel
/// carrying one of these finishes immediately after.
///
/// # The layout
///
/// ```text
/// [0][the file's bytes …]   the icon: every byte of the file, and no other
/// [1]                       the image ships no such icon
/// [2][error JSON …]         the icon could not be read, and why
/// [3]                       the file is larger than 1048576 bytes
/// ```
///
/// # How it is decided
///
/// The provider reads the image's document first, as an info channel
/// does, and then the file the document names. In order:
///
/// 1. The image ships no document: [`Missing`](Self::Missing).
/// 2. The document is larger than
///    [`MAX_BYTES`](crate::shared::containers::info::MAX_BYTES), is
///    not UTF-8, or breaks a rule of
///    [`Document::parse`](crate::shared::containers::info::Document::parse):
///    [`Error`](Self::Error).
/// 3. The document's `icons` has no member for this channel's icon:
///    `Missing`.
/// 4. The file the member names is not in the image's `/.diverge`, or
///    is not a regular file: `Missing`.
/// 5. The file is larger than `MAX_BYTES`:
///    [`TooLarge`](Self::TooLarge).
/// 6. Otherwise [`Icon`](Self::Icon), the file's bytes.
#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    /// The icon. Tag `0`.
    ///
    /// Everything after the tag is the file, byte for byte as the
    /// image holds it: no length before it, no encoding over it, and
    /// nothing after it. It is at most `1048576` bytes, and may be
    /// none at all, for a file that is empty. The provider has not
    /// read it: whether it is the kind of image its member says is
    /// the reader's to find out.
    Icon(Vec<u8>),
    /// The image ships no such icon. Tag `1`.
    ///
    /// Bare: there is nothing to carry. A fact about the image and
    /// not a failure: it ships no document, its document names no
    /// such icon, or the file named is not there.
    Missing,
    /// A failure. Tag `2`.
    ///
    /// The document is there and is not one, or is too large to be
    /// read, or the image could not be read. See
    /// [`shared::error::Error`](crate::shared::error::Error) for why
    /// it says so little.
    Error(Error),
    /// The icon's file is larger than
    /// [`MAX_BYTES`](crate::shared::containers::info::MAX_BYTES),
    /// `1048576` bytes. Tag `3`.
    ///
    /// Bare: there is nothing to carry. The provider measured the file
    /// as the image holds it; no part of it is answered, here or on
    /// any other frame.
    TooLarge,
}

/// Tag for [`Frame::Icon`].
const ICON: u8 = 0;
/// Tag for [`Frame::Missing`].
const MISSING: u8 = 1;
/// Tag for [`Frame::Error`].
const ERROR: u8 = 2;
/// Tag for [`Frame::TooLarge`].
const TOO_LARGE: u8 = 3;

impl Encode for Frame {
    /// The ordinary JSON failure, from the one half that has one.
    type Error = serde_json::Error;

    // Spelled out rather than `Self::Error`: this enum has a variant
    // called `Error`, so the associated type is ambiguous by that name.
    fn encode(&self, out: &mut Writer<'_>) -> Result<(), serde_json::Error> {
        match self {
            Frame::Icon(bytes) => {
                out.extend_from_slice(&[ICON]);
                out.extend_from_slice(bytes);
                Ok(())
            }
            Frame::Missing => {
                out.extend_from_slice(&[MISSING]);
                Ok(())
            }
            Frame::Error(error) => {
                out.extend_from_slice(&[ERROR]);
                error.encode(out)
            }
            Frame::TooLarge => {
                out.extend_from_slice(&[TOO_LARGE]);
                Ok(())
            }
        }
    }
}

impl Decode<'_> for Frame {
    /// Three ways to fail, and one of them is JSON.
    type Error = FrameError;

    // Spelled out for the same reason as `encode` above.
    fn decode(bytes: &[u8]) -> Result<Self, FrameError> {
        let (tag, rest) = bytes.split_first().ok_or(FrameError::Empty)?;
        match *tag {
            ICON => Ok(Frame::Icon(rest.to_vec())),
            MISSING => Ok(Frame::Missing),
            ERROR => Error::decode(rest).map(Frame::Error).map_err(FrameError::Error),
            TOO_LARGE => Ok(Frame::TooLarge),
            tag => Err(FrameError::UnknownTag(tag)),
        }
    }
}

/// An icon answer that could not be read.
#[derive(Debug)]
pub enum FrameError {
    /// No bytes at all, so not even a tag.
    Empty,
    /// A tag that is none of this frame's four.
    UnknownTag(u8),
    /// The error did not parse.
    Error(serde_json::Error),
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameError::Empty => f.write_str("icon answer frame is empty"),
            FrameError::UnknownTag(tag) => write!(f, "unknown icon answer frame tag {tag}"),
            FrameError::Error(error) => write!(f, "icon error did not parse: {error}"),
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
