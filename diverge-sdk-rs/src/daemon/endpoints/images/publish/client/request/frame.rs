//! What a client's request frame carries for an image publish.

use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};
use serde::{Deserialize, Deserializer, Serialize};

use crate::daemon::agent_log::Identity;
use crate::shared::containers::request::Reference;
use crate::shared::description::Description;
use crate::shared::relay::Relay;

use super::{Credential, Part};

/// Ask the daemon to publish one or more images to a registry as one
/// image. See [`publish`](super::super::super) for what the daemon
/// does with it and what the request is judged by.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct Frame {
    /// The images, and the platforms taken of each: one or more. One
    /// that takes every platform its manifest names is published as it
    /// is; anything else is merged into one new image first. A
    /// platform two of them take is `platform_repeated`. A list that
    /// is empty does not parse.
    #[serde(deserialize_with = "parts")]
    pub images: Vec<Part>,
    /// Where it goes: the registry, and the repository path there. See
    /// [`Reference`]. The registry is one the provider the publish is
    /// done on uses.
    pub reference: Reference,
    /// The tags to put on it there, each a tag as a registry takes
    /// one: one to 128 of letters, digits, `_`, `.` and `-`, the first
    /// not `.` or `-`. Absent or empty, the image is published by its
    /// digest alone.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// What to write to the registry as: see [`Credential`]. Absent,
    /// the registry is asked as nobody, which a registry that takes
    /// writes from nobody allows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential: Option<Credential>,
    /// What an image this request makes in the Store is kept as,
    /// beside its digest: one short line of plain text. Absent, none.
    /// Not used where the image was in the Store already.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    /// The provider to do it on: see [`Identity`]. Absent, the daemon
    /// picks one. Named, it must be one that could have been picked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<Identity>,
    /// Served by another daemon: see [`Relay`]. Absent, served here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relay: Option<Relay>,
}

/// One or more parts: a list that is empty does not parse.
fn parts<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Part>, D::Error> {
    let parts = Vec::<Part>::deserialize(deserializer)?;
    if parts.is_empty() {
        return Err(serde::de::Error::custom("a publish names one or more images"));
    }
    Ok(parts)
}

/// This frame's tag among the scope-opening requests.
///
/// One byte at the front of the payload, which is what tells a reader
/// which request it holds. See the table in
/// [`endpoints`](crate::daemon::endpoints) for the whole allocation.
const TAG: u8 = 164;

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

/// An images publish request frame that could not be read.
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
            FrameError::Empty => f.write_str("images publish request frame is empty"),
            FrameError::UnexpectedTag(tag) => write!(f, "expected images publish request tag {TAG}, found {tag}"),
            FrameError::Body(error) => write!(f, "images publish request did not parse: {error}"),
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
