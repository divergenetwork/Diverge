//! Opening the scope.

use super::super::request;
use super::super::super::server::response;
use super::Responder;
use crate::daemon::client::stream;
use crate::wire::client::handle::Handle;
use crate::wire::decode::Decode as _;

/// A requests listen that did not open.
pub type OpenError = stream::OpenError<serde_json::Error>;

/// The stream of a listen: one [`response::Frame`] per response, for
/// as long as the scope is open.
pub type ExecuteStream = stream::ExecuteStream<response::Frame, response::FrameError>;

/// Open the listen, and hand back the stream of what the caller is
/// sent and the [`Responder`] it answers and ends it with. Nothing is
/// read here: the first frame may be a refusal — see
/// [`response::Frame`] — and is the stream's first item.
pub async fn execute(handle: &Handle, request: &request::Frame) -> Result<(ExecuteStream, Responder), OpenError> {
    let (stream, cancel) = stream::execute_cancellable(handle, request, |_, payload| response::Frame::decode(payload)).await?;
    Ok((stream, Responder::new(handle.clone(), cancel)))
}
