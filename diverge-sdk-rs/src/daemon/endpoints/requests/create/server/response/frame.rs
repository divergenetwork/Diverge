//! What a server's response frame carries for a requests create.

use serde::Serialize;
use std::fmt;

use super::{Created, Delivered, Responded};
use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};
use crate::shared::error::Error;

/// What becomes of a request, or the one refusal that ends the create.
///
/// A create is a stream with an end: [`Created`](Self::Created),
/// first; zero or more [`Delivered`](Self::Delivered), one for each
/// listener sent it, as each is; then
/// [`Responded`](Self::Responded) and the finish — or, where the
/// caller took the request back or went, the finish with no
/// `Responded` before it. A create that is refused is exactly one of
/// [`Forbidden`](Self::Forbidden), [`NoRecipient`](Self::NoRecipient) and
/// [`Error`](Self::Error), with nothing before it, then the finish.
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
    /// The request exists. Tag `0`.
    Created(Created),
    /// A listener was sent it. Tag `1`.
    Delivered(Delivered),
    /// It was answered. Tag `2`.
    Responded(Responded),
    /// The caller holds no `request` grant: over accounts, over
    /// agents, over standalone tools, or over dependency tools. Tag `3`.
    Forbidden,
    /// The request is made of nobody: no account, no agent and no tool there is
    /// is both one the request addresses and one the caller's
    /// `request` grants reach. Tag `4`.
    NoRecipient,
    /// A failure. Tag `5`.
    Error(Error),
}

/// Tag for [`Frame::Created`].
const CREATED: u8 = 0;

/// Tag for [`Frame::Delivered`].
const DELIVERED: u8 = 1;

/// Tag for [`Frame::Responded`].
const RESPONDED: u8 = 2;

/// Tag for [`Frame::Forbidden`].
const FORBIDDEN: u8 = 3;

/// Tag for [`Frame::NoRecipient`].
const NO_RECIPIENT: u8 = 4;

/// Tag for [`Frame::Error`].
const ERROR: u8 = 5;

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
            Frame::Created(value) => {
                out.extend_from_slice(&[CREATED]);
                serde_json::to_writer(out, value)
            }
            Frame::Delivered(value) => {
                out.extend_from_slice(&[DELIVERED]);
                serde_json::to_writer(out, value)
            }
            Frame::Responded(value) => {
                out.extend_from_slice(&[RESPONDED]);
                serde_json::to_writer(out, value)
            }
            Frame::Forbidden => bare(out, FORBIDDEN),
            Frame::NoRecipient => bare(out, NO_RECIPIENT),
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
            CREATED => serde_json::from_slice(rest).map(Frame::Created).map_err(FrameError::Body),
            DELIVERED => serde_json::from_slice(rest).map(Frame::Delivered).map_err(FrameError::Body),
            RESPONDED => serde_json::from_slice(rest).map(Frame::Responded).map_err(FrameError::Body),
            FORBIDDEN => Ok(Frame::Forbidden),
            NO_RECIPIENT => Ok(Frame::NoRecipient),
            ERROR => Error::decode(rest).map(Frame::Error).map_err(FrameError::Error),
            tag => Err(FrameError::UnknownTag(tag)),
        }
    }
}

/// A response frame of a requests create that could not be read.
#[derive(Debug)]
pub enum FrameError {
    /// No bytes at all, so not even a tag.
    Empty,
    /// A tag that is none of this frame's six.
    UnknownTag(u8),
    /// What the tag promised did not parse.
    Body(serde_json::Error),
    /// The error did not parse.
    Error(serde_json::Error),
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameError::Empty => f.write_str("the response frame of a requests create is empty"),
            FrameError::UnknownTag(tag) => write!(f, "unknown response frame tag {tag} of a requests create"),
            FrameError::Body(error) => write!(f, "an answer of a requests create did not parse: {error}"),
            FrameError::Error(error) => write!(f, "an error of a requests create did not parse: {error}"),
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
