//! What a client's channel request frame carries on a requests listen.

use std::fmt;

use super::Respond;
use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};

/// What a client asks the daemon for while a listen is open.
///
/// A payload leads with one byte saying which, and the rest is that
/// variant's own bytes.
///
/// | tag | asks for |
/// |-----|----------|
/// | `0` | [`Cancel`](Self::Cancel) |
/// | `1` | [`Respond`](Self::Respond) |
#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    /// Stop listening. Tag `0`.
    ///
    /// Carries nothing, and nothing comes back on this channel: the
    /// daemon sends no channel response and no channel response
    /// finish on it. What comes back is the scope's finish, after
    /// every answer still being given has been. This is how a listen,
    /// which never ends on its own, is ended.
    Cancel,
    /// Answer a request. Tag `1`, then the [`Respond`] as JSON.
    ///
    /// Answered with exactly one
    /// [`Delivery`](crate::daemon::endpoints::requests::listen::server::channel_response::Frame)
    /// — whether the answer was taken — then the finish.
    Respond(Respond),
}

/// Tag for [`Frame::Cancel`].
const CANCEL: u8 = 0;

/// Tag for [`Frame::Respond`].
const RESPOND: u8 = 1;

/// A tag, then the answer's JSON; the cancel is the tag alone.
impl Encode for Frame {
    /// The ordinary JSON failure, from the one variant that has one.
    type Error = serde_json::Error;

    fn encode(&self, out: &mut Writer<'_>) -> Result<(), Self::Error> {
        match self {
            Frame::Cancel => {
                out.extend_from_slice(&[CANCEL]);
                Ok(())
            }
            Frame::Respond(respond) => {
                out.extend_from_slice(&[RESPOND]);
                serde_json::to_writer(out, respond)
            }
        }
    }
}

impl Decode<'_> for Frame {
    /// Three ways to fail: nothing, a tag that is neither, or an
    /// answer that did not parse.
    type Error = FrameError;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        let (tag, rest) = bytes.split_first().ok_or(FrameError::Empty)?;
        match *tag {
            CANCEL => Ok(Frame::Cancel),
            RESPOND => serde_json::from_slice(rest).map(Frame::Respond).map_err(FrameError::Respond),
            tag => Err(FrameError::UnknownTag(tag)),
        }
    }
}

/// A requests listen channel request that could not be read.
#[derive(Debug)]
pub enum FrameError {
    /// No bytes at all, so not even a tag.
    Empty,
    /// A tag that is neither of this frame's two.
    UnknownTag(u8),
    /// The answer did not parse.
    Respond(serde_json::Error),
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameError::Empty => f.write_str("requests listen channel request frame is empty"),
            FrameError::UnknownTag(tag) => write!(f, "unknown requests listen channel request tag {tag}"),
            FrameError::Respond(error) => write!(f, "requests listen answer did not parse: {error}"),
        }
    }
}

impl std::error::Error for FrameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FrameError::Respond(error) => Some(error),
            FrameError::Empty | FrameError::UnknownTag(_) => None,
        }
    }
}
