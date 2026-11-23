//! What a client's request frame carries for an images merge.

use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};
use serde::{Deserialize, Deserializer, Serialize};

use crate::daemon::agent_log::Identity;
use crate::shared::relay::Relay;

use super::Part;

/// Ask the daemon to make one image of one or more, in its Store on a
/// provider. See [`merge`](super::super::super) for where the images
/// come from and what the request is judged by.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct Frame {
    /// The provider the image is made on, and whose Store of the
    /// daemon's it goes into: see [`Identity`].
    pub provider: Identity,
    /// The images, and the platforms taken of each: one or more. A
    /// platform two of them take is `platform_repeated`. A list that
    /// is empty does not parse.
    #[serde(deserialize_with = "parts")]
    pub images: Vec<Part>,
    /// Served by another daemon: see [`Relay`]. Absent, served here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relay: Option<Relay>,
}

/// The images, one or more.
fn parts<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Part>, D::Error> {
    let parts = Vec::<Part>::deserialize(deserializer)?;
    if parts.is_empty() {
        return Err(serde::de::Error::custom("a merge names one image or more"));
    }
    Ok(parts)
}

/// This frame's tag among the scope-opening requests.
///
/// One byte at the front of the payload, which is what tells a reader
/// which request it holds. See the table in
/// [`endpoints`](crate::daemon::endpoints) for the whole allocation.
const TAG: u8 = 167;

/// JSON, as every request of the daemon's is.
impl Encode for Frame {
    /// The ordinary JSON failure. The tag cannot fail.
    type Error = serde_json::Error;

    fn encode(&self, out: &mut Writer<'_>) -> Result<(), Self::Error> {
        out.extend_from_slice(&[TAG]);
        serde_json::to_writer(out, self)
    }
}

impl Decode<'_> for Frame {
    /// Three ways to fail, and only one of them is JSON.
    type Error = FrameError;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        let (tag, rest) = bytes.split_first().ok_or(FrameError::Empty)?;
        if *tag != TAG {
            return Err(FrameError::UnexpectedTag(*tag));
        }
        serde_json::from_slice(rest).map_err(FrameError::Body)
    }
}

/// A request frame of an images merge that could not be read.
#[derive(Debug)]
pub enum FrameError {
    /// No bytes at all, so not even a tag.
    Empty,
    /// A tag naming some other request.
    UnexpectedTag(u8),
    /// The request did not parse.
    Body(serde_json::Error),
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameError::Empty => f.write_str("the request frame of an images merge is empty"),
            FrameError::UnexpectedTag(tag) => write!(f, "expected the request tag {TAG} of an images merge, found {tag}"),
            FrameError::Body(error) => write!(f, "the request of an images merge did not parse: {error}"),
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
