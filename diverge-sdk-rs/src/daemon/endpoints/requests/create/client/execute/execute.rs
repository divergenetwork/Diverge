//! Opening the scope.

use super::super::request;
use super::super::super::server::response;
use crate::daemon::client::{stream, Cancel};
use crate::wire::client::handle::Handle;
use crate::wire::decode::Decode as _;

/// A requests create that did not open.
pub type OpenError = stream::OpenError<serde_json::Error>;

/// The stream of a create: one [`response::Frame`] per response,
/// ended by the daemon's finish.
pub type ExecuteStream = stream::ExecuteStream<response::Frame, response::FrameError>;

/// Make the request, and hand back the stream of what becomes of it —
/// created, each delivery, then its answer — and the [`Cancel`] that
/// takes it back. Nothing is read here.
pub async fn execute(handle: &Handle, request: &request::Frame) -> Result<(ExecuteStream, Cancel), OpenError> {
    stream::execute_cancellable(handle, request, |_, payload| response::Frame::decode(payload)).await
}
