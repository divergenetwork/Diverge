//! What a server's response frame carries for a requests listen.

use serde::Serialize;
use std::fmt;

use super::{Asked, Withdrawn};
use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};
use crate::shared::error::Error;

/// A request, a withdrawal, or the one refusal that ends the listen.
///
/// A listen is a stream with no end of its own: zero or more
/// [`Request`](Self::Request) and [`Withdrawn`](Self::Withdrawn), in
/// the order they happen, until the caller ends it. A listen that is
/// not opened is exactly one of [`AlreadyOpen`](Self::AlreadyOpen),
/// [`Forbidden`](Self::Forbidden) and [`Error`](Self::Error), with
/// nothing before it, then the finish.
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
    /// A request made of the caller. Tag `0`.
    Request(Asked),
    /// A request is wanted no more. Tag `1`.
    Withdrawn(Withdrawn),
    /// The caller has a listen open already, on this connection or
    /// another, and this one is not opened. Tag `2`.
    AlreadyOpen,
    /// The account the request is served for is none there is. Tag
    /// `3`.
    Forbidden,
    /// A failure. Tag `4`.
    Error(Error),
}

/// Tag for [`Frame::Request`].
const REQUEST: u8 = 0;

/// Tag for [`Frame::Withdrawn`].
const WITHDRAWN: u8 = 1;

/// Tag for [`Frame::AlreadyOpen`].
const ALREADY_OPEN: u8 = 2;

/// Tag for [`Frame::Forbidden`].
const FORBIDDEN: u8 = 3;

/// Tag for [`Frame::Error`].
const ERROR: u8 = 4;

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
            Frame::Request(value) => {
                out.extend_from_slice(&[REQUEST]);
                serde_json::to_writer(out, value)
            }
            Frame::Withdrawn(value) => {
                out.extend_from_slice(&[WITHDRAWN]);
                serde_json::to_writer(out, value)
            }
            Frame::AlreadyOpen => bare(out, ALREADY_OPEN),
            Frame::Forbidden => bare(out, FORBIDDEN),
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
            REQUEST => serde_json::from_slice(rest).map(Frame::Request).map_err(FrameError::Body),
            WITHDRAWN => serde_json::from_slice(rest).map(Frame::Withdrawn).map_err(FrameError::Body),
            ALREADY_OPEN => Ok(Frame::AlreadyOpen),
            FORBIDDEN => Ok(Frame::Forbidden),
            ERROR => Error::decode(rest).map(Frame::Error).map_err(FrameError::Error),
            tag => Err(FrameError::UnknownTag(tag)),
        }
    }
}

/// A response frame of a requests listen that could not be read.
#[derive(Debug)]
pub enum FrameError {
    /// No bytes at all, so not even a tag.
    Empty,
    /// A tag that is none of this frame's five.
    UnknownTag(u8),
    /// What the tag promised did not parse.
    Body(serde_json::Error),
    /// The error did not parse.
    Error(serde_json::Error),
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameError::Empty => f.write_str("the response frame of a requests listen is empty"),
            FrameError::UnknownTag(tag) => write!(f, "unknown response frame tag {tag} of a requests listen"),
            FrameError::Body(error) => write!(f, "an answer of a requests listen did not parse: {error}"),
            FrameError::Error(error) => write!(f, "an error of a requests listen did not parse: {error}"),
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
