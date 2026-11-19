//! Whether an answer was taken.

use std::fmt;

use serde::Serialize;

use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};

/// Whether the daemon took a listener's answer as the request's:
/// exactly one of these on the answer's channel, then the finish. One
/// byte, `0` or `1`. As JSON, for a client that prints it, the
/// variant's name in snake case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Frame {
    /// The answer is the request's: its creator is sent it, with who
    /// gave it, and the request is over. Every other listener it was
    /// sent to is told it is withdrawn; this one is not, and takes
    /// this as that word. Tag `0`.
    Delivered,
    /// The answer was not taken, and changes nothing: the request was
    /// answered by another listener, was withdrawn, was never sent to
    /// this listener, or is none there is. Tag `1`.
    NotDelivered,
}

/// Tag for [`Frame::Delivered`].
const DELIVERED: u8 = 0;

/// Tag for [`Frame::NotDelivered`].
const NOT_DELIVERED: u8 = 1;

impl Encode for Frame {
    /// [`Infallible`](std::convert::Infallible): a known byte.
    type Error = std::convert::Infallible;

    fn encode(&self, out: &mut Writer<'_>) -> Result<(), Self::Error> {
        out.extend_from_slice(&[match self {
            Frame::Delivered => DELIVERED,
            Frame::NotDelivered => NOT_DELIVERED,
        }]);
        Ok(())
    }
}

impl Decode<'_> for Frame {
    type Error = FrameError;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        match bytes.first() {
            None => Err(FrameError::Empty),
            Some(&DELIVERED) => Ok(Frame::Delivered),
            Some(&NOT_DELIVERED) => Ok(Frame::NotDelivered),
            Some(&tag) => Err(FrameError::UnknownTag(tag)),
        }
    }
}

/// A delivery that could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameError {
    /// No bytes at all, so not even a tag.
    Empty,
    /// A tag that is neither of this frame's two.
    UnknownTag(u8),
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameError::Empty => f.write_str("requests listen delivery frame is empty"),
            FrameError::UnknownTag(tag) => write!(f, "unknown requests listen delivery tag {tag}"),
        }
    }
}

impl std::error::Error for FrameError {}
