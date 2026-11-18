//! Opening the scope.

use super::super::request;
use super::super::super::server::response;
use crate::daemon::client::stream;
use crate::wire::client::handle::Handle;
use crate::wire::decode::Decode as _;
use crate::wire::encode::{Encode as _, Writer};

/// An images publish that did not open.
pub type OpenError = stream::OpenError<serde_json::Error>;

/// The stream of a publish: one [`response::Frame`] per response,
/// ended by the daemon's finish.
pub type ExecuteStream = stream::ExecuteStream<response::Frame, response::FrameError>;

/// Ask the daemon to publish, and hand back the stream of what it
/// answers, without reading anything: what happened as it happened,
/// then the one answer that ends it — see [`response::Frame`] — then
/// the daemon's finish. The daemon opens no channel on the scope, and
/// there is no cancel.
pub async fn execute(handle: &Handle, request: &request::Frame) -> Result<ExecuteStream, OpenError> {
    let mut payload = Vec::new();
    request.encode(&mut Writer::new(&mut payload)).map_err(stream::OpenError::Request)?;
    let scope = handle.send_request(&payload).await.map_err(stream::OpenError::Send)?;
    Ok(stream::ExecuteStream::new(scope.response_receiver, |_, payload| response::Frame::decode(payload)))
}
