//! Answering on a listen, and ending it.

use std::fmt;

use super::super::channel_request::{self, Respond};
use super::super::super::server::channel_response;
use crate::daemon::client::Cancel;
use crate::daemon::endpoints::requests::RequestId;
use crate::wire::client::handle::{Handle, SendError};
use crate::wire::decode::Decode as _;
use crate::wire::encode::{Encode as _, Writer};
use crate::wire::frame;

/// What a listener does on its open listen: answer a request, and
/// stop listening. Cloned freely; every answer is a channel of its
/// own, so several may be given at once.
#[derive(Debug, Clone)]
pub struct Responder {
    handle: Handle,
    cancel: Cancel,
}

impl Responder {
    pub(crate) fn new(handle: Handle, cancel: Cancel) -> Self {
        Responder { handle, cancel }
    }

    /// Answer the request `id` with `response`: whether the daemon
    /// took the answer as the request's. `NotDelivered` is an answer
    /// like the other: the request was answered or withdrawn first,
    /// or is none this listener was sent.
    pub async fn respond(&self, id: RequestId, response: Vec<rmcp::model::ContentBlock>) -> Result<channel_response::Frame, RespondError> {
        let mut payload = Vec::new();
        channel_request::Frame::Respond(Respond { id, response })
            .encode(&mut Writer::new(&mut payload))
            .map_err(RespondError::Request)?;
        let mut channel = self.handle.send_channel_request(self.cancel.scope(), &payload).await.map_err(RespondError::Send)?;
        let bytes = channel.response_receiver.recv().await.ok_or(RespondError::Closed)?;
        match frame::server::ServerFrame::decode(&bytes).map_err(RespondError::Frame)? {
            frame::server::ServerFrame::ChannelResponse { payload, .. } => channel_response::Frame::decode(payload).map_err(RespondError::Response),
            _ => Err(RespondError::Unanswered),
        }
    }

    /// Stop listening: the scope finishes, which ends the stream.
    pub async fn cancel(&self) -> Result<(), SendError> {
        self.cancel.cancel().await
    }
}

/// An answer that was not put, or whose delivery was not read. None of
/// these says the answer was not taken: only a
/// [`channel_response::Frame`] says that.
#[derive(Debug)]
pub enum RespondError {
    /// The answer did not serialize.
    Request(serde_json::Error),
    /// The channel could not be opened.
    Send(SendError),
    /// The connection ended before the daemon answered.
    Closed,
    /// The daemon's frame did not decode.
    Frame(frame::FrameError),
    /// The channel finished with nothing before it.
    Unanswered,
    /// The delivery did not decode.
    Response(channel_response::FrameError),
}

impl fmt::Display for RespondError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RespondError::Request(error) => write!(f, "the answer did not serialize: {error}"),
            RespondError::Send(error) => write!(f, "the answer never went out: {error}"),
            RespondError::Closed => f.write_str("the connection ended before the answer's delivery was said"),
            RespondError::Frame(error) => write!(f, "the answer's delivery did not decode: {error}"),
            RespondError::Unanswered => f.write_str("the daemon finished the answer's channel without saying whether it was delivered"),
            RespondError::Response(error) => write!(f, "the answer's delivery did not parse: {error}"),
        }
    }
}

impl std::error::Error for RespondError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RespondError::Request(error) => Some(error),
            RespondError::Send(error) => Some(error),
            RespondError::Frame(error) => Some(error),
            RespondError::Response(error) => Some(error),
            RespondError::Closed | RespondError::Unanswered => None,
        }
    }
}
