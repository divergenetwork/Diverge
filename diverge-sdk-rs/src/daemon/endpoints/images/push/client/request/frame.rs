//! What a client's request frame carries for an image push.

use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};
use serde::{Deserialize, Serialize};

use crate::daemon::agent_log::Identity;
use crate::shared::containers::request::Image;
use crate::shared::relay::Relay;

use super::Source;

/// Ask the daemon to push an image into its Store on a provider: the
/// provider's
/// [`images::push`](crate::provider::endpoints::images::push), asked
/// by the daemon as itself. See [`push`](super::super::super) for
/// where the image comes from and what the request is judged by.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct Frame {
    /// The provider to push to, and whose Store of the daemon's the
    /// image goes into: see [`Identity`].
    pub provider: Identity,
    /// The image: its digest, which is the image, and the references
    /// under which its bytes may be fetched. A reference naming a
    /// registry the provider uses lets the provider fetch it there;
    /// with none that does, the source is asked for it.
    pub image: Image,
    /// Who is asked for the image where the provider asks: the client,
    /// or the daemon, which finds a source itself. See [`Source`].
    /// Required.
    pub source: Source,
    /// The platforms to hold of the image: one or more, each once,
    /// each a [platform string](crate::shared::platform::of) such as
    /// `linux/amd64`. Required. Every one must be a platform the image
    /// has, or the push is `platform_not_found` and nothing is added.
    /// A provider holds a platform it could not run, and a push of a
    /// digest the daemon owns there already adds the platforms it
    /// lacked.
    #[serde(deserialize_with = "crate::shared::platform::some")]
    pub platforms: Vec<String>,
    /// Served by another daemon: see [`Relay`]. Absent, served here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relay: Option<Relay>,
}

/// This frame's tag among the scope-opening requests.
///
/// One byte at the front of the payload, which is what tells a reader
/// which request it holds. The frame layer does not discriminate them —
/// [`ClientFrame::Request`](crate::wire::frame::client::ClientFrame::Request)
/// is one type carrying bytes — so the distinction has to be in the
/// bytes, and each request owns the value that names it.
///
/// See the table in [`endpoints`](crate::daemon::endpoints) for the
/// whole allocation. The values are chosen across modules that do not
/// know about each other, so the table is the only place they can be
/// seen at once.
const TAG: u8 = 160;

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

/// An images push request frame that could not be read.
#[derive(Debug)]
pub enum FrameError {
    /// No bytes at all, so not even a tag.
    Empty,
    /// A tag naming some other request.
    ///
    /// A reader that dispatched on the tag will not see this. One that
    /// assumed which request it held, and was wrong, will — which is
    /// the point of checking a tag rather than skipping it.
    UnexpectedTag(u8),
    /// The request did not parse.
    Body(serde_json::Error),
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameError::Empty => f.write_str("images push request frame is empty"),
            FrameError::UnexpectedTag(tag) => {
                write!(f, "expected images push request tag {TAG}, found {tag}")
            }
            FrameError::Body(error) => {
                write!(f, "images push request did not parse: {error}")
            }
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
