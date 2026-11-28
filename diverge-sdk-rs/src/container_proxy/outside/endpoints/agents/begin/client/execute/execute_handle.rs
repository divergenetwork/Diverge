//! The begin scope, held for the connection's life.

use super::super::channel_request;
use crate::wire::client::handle::{Handle, SendError};
use crate::wire::encode::{Encode, Writer};
use crate::provider::endpoints::containers::client::answered::{Daemon, Enqueue, Postgres};
use crate::provider::endpoints::containers::client::{ChannelStream, OpenError, UnaryError, unary};
use crate::shared::containers::{daemon, enqueue, postgres};

/// The begin scope an agent container's server holds.
///
/// Every channel the server may open on it is a method here — the
/// image's DIVERGE.md, a message for the agent, the
/// server's half of a database connection and of a daemon
/// connection — and so is answering a
/// channel the proxy opened, by the proxy's own channel number.
/// Clones share the scope.
///
/// # Dropping it does nothing
///
/// No destructor sends a frame. The scope is the connection's life,
/// and the connection ends when the container does.
#[derive(Debug, Clone)]
pub struct ExecuteHandle {
    handle: Handle,
    scope: u32,
}

/// One channel request of this family, as its bytes.
fn payload(frame: &channel_request::Frame) -> Result<Vec<u8>, serde_json::Error> {
    let mut bytes = Vec::new();
    frame.encode(&mut Writer::new(&mut bytes))?;
    Ok(bytes)
}

impl ExecuteHandle {
    pub(super) fn new(handle: Handle, scope: u32) -> Self {
        ExecuteHandle { handle, scope }
    }

    /// The scope's number, as this end minted it.
    pub fn scope(&self) -> u32 {
        self.scope
    }


    /// A batch of messages for the agent, in order — starting a loop
    /// when none runs, handed to the program's queue when one does —
    /// answered with its fate, every message's alike, whenever that
    /// is known; nothing times it out. The order is the batch's: see
    /// [`enqueue::request::Request`].
    ///
    /// The request is whole: its `index` among the scope's enqueues,
    /// and its item — a batch of chat messages, which the lines above
    /// are about, or one structured decision, which waits for the
    /// agent to be idle and starts a run of its own.
    pub async fn enqueue(&self, request: enqueue::request::Request) -> Result<enqueue::response::Frame, UnaryError<Enqueue>> {
        let payload = payload(&channel_request::Frame::Enqueue(request))
            .map_err(UnaryError::Request)?;
        unary::<Enqueue>(&self.handle, self.scope, &payload).await
    }

    /// The server's half of a daemon connection the proxy announced,
    /// quoting the id the proxy minted: what comes back is every client
    /// frame the program sent on it, until the finish, which is the
    /// program's socket ended. See
    /// [`daemon`](crate::shared::containers::daemon).
    pub async fn daemon(&self, connection_id: u32) -> Result<ChannelStream<Daemon>, OpenError> {
        let payload = payload(&channel_request::Frame::Daemon(daemon::request::Daemon { connection_id }))
            .map_err(OpenError::Request)?;
        let channel = self
            .handle
            .send_channel_request(self.scope, &payload)
            .await
            .map_err(OpenError::Send)?;
        Ok(ChannelStream::new(channel.response_receiver))
    }

    /// The server's half of a database connection the proxy
    /// announced, quoting the id the proxy minted: what comes back is
    /// everything the container's driver wrote, until the finish.
    pub async fn postgres(&self, connection_id: u32) -> Result<ChannelStream<Postgres>, OpenError> {
        let payload = payload(&channel_request::Frame::Postgres(postgres::request::Postgres { connection_id }))
            .map_err(OpenError::Request)?;
        let channel = self
            .handle
            .send_channel_request(self.scope, &payload)
            .await
            .map_err(OpenError::Send)?;
        Ok(ChannelStream::new(channel.response_receiver))
    }

    /// One answer on a channel the proxy opened, by its number.
    pub async fn respond(&self, channel: u32, payload: &[u8]) -> Result<(), SendError> {
        self.handle.send_channel_response(self.scope, channel, payload).await
    }

    /// The end of the answers on a channel the proxy opened.
    pub async fn finish(&self, channel: u32) -> Result<(), SendError> {
        self.handle.send_channel_response_finish(self.scope, channel).await
    }
}
