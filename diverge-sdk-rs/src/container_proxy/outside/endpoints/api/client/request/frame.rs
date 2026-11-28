//! What a client's request frame carries for an API request.

use crate::shared::containers::api;
use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};

/// Open an API request: the tag, then its head as JSON.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Frame(pub api::request::Head);

/// This frame's tag among the scope-opening requests.
const TAG: u8 = 10;

impl Encode for Frame {
    /// The ordinary JSON failure.
    type Error = serde_json::Error;

    fn encode(&self, out: &mut Writer<'_>) -> Result<(), Self::Error> {
        out.extend_from_slice(&[TAG]);
        self.0.encode(out)
    }
}

impl Decode<'_> for Frame {
    /// See [`FrameError`].
    type Error = FrameError;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        let (tag, rest) = bytes.split_first().ok_or(FrameError::Empty)?;
        if *tag != TAG {
            return Err(FrameError::UnexpectedTag(*tag));
        }
        api::request::Head::decode(rest).map(Frame).map_err(FrameError::Body)
    }
}

/// An API request that did not decode.
#[derive(Debug)]
pub enum FrameError {
    /// No tag at all.
    Empty,
    /// Another endpoint's tag.
    UnexpectedTag(u8),
    /// The head's JSON did not parse.
    Body(serde_json::Error),
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameError::Empty => f.write_str("API request frame is empty"),
            FrameError::UnexpectedTag(tag) => write!(f, "expected API request tag {TAG}, found {tag}"),
            FrameError::Body(error) => write!(f, "API request did not parse: {error}"),
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
