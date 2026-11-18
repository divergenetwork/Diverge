//! Asking and reading the answer, in one call.

use std::fmt;

use super::super::request;
use crate::provider::endpoints::images::merge::server::response::{self, Merged};
use crate::shared::error::Error;
use crate::wire::client::handle::{Handle, SendError};
use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};
use crate::wire::frame;

/// Merge the platforms named into one new image, and wait for the
/// answer.
///
/// One question, one answer, then the scope is over. The answer is
/// the image merged, or the digest the caller owned already — both
/// are the image there, and [`Outcome`] tells them apart. A source the
/// caller does not own, a platform a source does not hold, a platform
/// named twice, a Store with no room and a provider with no Store are
/// the protocol's defined refusals, not failures. Everything else is
/// an [`ExecuteError::Provider`].
///
/// # It reads one frame and leaves
///
/// There is no timeout here or anywhere else in this protocol: a
/// provider that never answers is waited on until the connection
/// dies, at which point this returns [`ExecuteError::Closed`]. The
/// finish that follows the answer is never read; dropping the scope
/// is what tells the router nobody is listening for the rest.
pub async fn execute(handle: &Handle, request: &request::Frame) -> Result<Outcome, ExecuteError> {
    let mut payload = Vec::new();
    request.encode(&mut Writer::new(&mut payload)).map_err(ExecuteError::Request)?;
    let mut scope = handle.send_request(&payload).await.map_err(ExecuteError::Send)?;
    let bytes = scope.response_receiver.recv().await.ok_or(ExecuteError::Closed)?;
    let envelope = frame::server::ServerFrame::decode(&bytes).map_err(ExecuteError::Frame)?;
    // A router puts only a response and its finish on a scope's
    // response stream, so the only other thing this can be is a finish
    // arriving first — a scope that ended without saying anything.
    let frame::server::ServerFrame::Response { payload, .. } = envelope else {
        return Err(ExecuteError::Unanswered);
    };
    match response::Frame::decode(payload).map_err(ExecuteError::Response)? {
        response::Frame::Merged(merged) => Ok(Outcome::Merged(merged)),
        response::Frame::Exists(digest) => Ok(Outcome::Exists(digest)),
        response::Frame::NotFound => Err(ExecuteError::NotFound),
        response::Frame::PlatformNotHeld => Err(ExecuteError::PlatformNotHeld),
        response::Frame::PlatformRepeated => Err(ExecuteError::PlatformRepeated),
        response::Frame::InsufficientCapacity => Err(ExecuteError::InsufficientCapacity),
        response::Frame::Unsupported => Err(ExecuteError::Unsupported),
        response::Frame::Error(error) => Err(ExecuteError::Provider(error)),
    }
}

/// A merge that left the image in the caller's Store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The image was made by this merge.
    Merged(Merged),
    /// The caller owned the digest already, and nothing was added.
    Exists(String),
}

/// A merge that did not produce the image.
///
/// All but the last six are this end's view of something going wrong.
/// The last six are the provider saying so itself, and they are the
/// only ones that mean the exchange worked.
#[derive(Debug)]
pub enum ExecuteError {
    /// The request never went out. See [`SendError`].
    Send(SendError),
    /// The request would not serialize.
    Request(postcard::Error),
    /// The connection ended before anything came back.
    Closed,
    /// What came back was not a frame.
    Frame(frame::FrameError),
    /// The scope finished without an answer in it.
    Unanswered,
    /// The response frame did not parse.
    Response(response::FrameDecodeError),
    /// The caller owns no image by one of the digests. Nothing was
    /// made.
    NotFound,
    /// A source does not hold a platform taken from it. Nothing was
    /// made.
    PlatformNotHeld,
    /// A platform was named more than once. Nothing was made.
    PlatformRepeated,
    /// The Store has no room for the image. Nothing was made.
    InsufficientCapacity,
    /// The provider has no Store.
    Unsupported,
    /// The provider could not do it, and said so. See
    /// [`shared::error::Error`](crate::shared::error::Error) for why
    /// it says so little.
    Provider(Error),
}

impl fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecuteError::Send(error) => write!(f, "the request never went out: {error}"),
            ExecuteError::Request(error) => write!(f, "image merge request did not serialize: {error}"),
            ExecuteError::Closed => f.write_str("connection ended before the image merge answered"),
            ExecuteError::Frame(error) => write!(f, "image merge answer did not decode: {error}"),
            ExecuteError::Unanswered => f.write_str("image merge finished without an answer"),
            ExecuteError::Response(error) => write!(f, "image merge answer did not parse: {error}"),
            ExecuteError::NotFound => f.write_str("the caller owns no image by one of the digests"),
            ExecuteError::PlatformNotHeld => f.write_str("a source does not hold a platform taken from it"),
            ExecuteError::PlatformRepeated => f.write_str("a platform is named more than once"),
            ExecuteError::InsufficientCapacity => f.write_str("the provider has no room for the image"),
            ExecuteError::Unsupported => f.write_str("the provider has no Store"),
            ExecuteError::Provider(_) => f.write_str("the provider could not complete the image merge"),
        }
    }
}

impl std::error::Error for ExecuteError {
    /// [`Provider`](ExecuteError::Provider) has no source, because what
    /// it carries is not a Rust error and deliberately does not
    /// implement one.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ExecuteError::Send(error) => Some(error),
            ExecuteError::Request(error) => Some(error),
            ExecuteError::Frame(error) => Some(error),
            ExecuteError::Response(error) => Some(error),
            ExecuteError::Closed
            | ExecuteError::Unanswered
            | ExecuteError::NotFound
            | ExecuteError::PlatformNotHeld
            | ExecuteError::PlatformRepeated
            | ExecuteError::InsufficientCapacity
            | ExecuteError::Unsupported
            | ExecuteError::Provider(_) => None,
        }
    }
}
