//! The desk, as the agent reaches it: one stream for each turn, opened
//! on the desk's API through the daemon.
//!
//! The desk is the agent's own dependency tool, a container beside
//! it, where the person is. The agent holds nothing of the turn but
//! the stream: the transcript, the page and the person's acts are the
//! desk's. The lines on the stream are [`wire`](crate::wire)'s.
//!
//! The rules:
//!
//! 1. A turn is one request on the desk's API, upgraded: opened when a
//!    run starts, and answered `101` before the run is. A desk that
//!    answers anything else, or cannot be reached, is the run's
//!    refusal, with why.
//! 2. At most one turn is open. A run that comes while one is open is
//!    refused, unless the open one's run is gone — the proxy stopped
//!    reading it — which closes it.
//! 3. The run's messages go up first; the agent's tools, listed, go up
//!    after them, unasked.
//! 4. Messages enqueued while a turn is open go up at once, and the
//!    enqueue is answered then; with none open it is [`Closed`], and
//!    the proxy starts the next run on them.
//! 5. A tool call the desk asks for is made on a task of its own,
//!    through the proxy's MCP, and its result goes up under the call's
//!    id; a call that fails as a link goes up as an error result, so
//!    every call has its result.
//! 6. The stream is closed when the run is over, however it ends: the
//!    way up is let go, and what is still in flight on it goes first.

use std::sync::Arc;

use diverge_sdk::container_proxy::inside::Client;
use diverge_sdk::daemon::endpoints::dependency_tools::api;
use diverge_sdk::shared::containers::api::request::Head;
use diverge_sdk::wire::client::handle::Handle;
use futures_util::StreamExt as _;
use rmcp::model::{CallToolRequestParams, CallToolResult, ContentBlock, JsonObject};
use tokio::sync::{Mutex, mpsc};

use crate::register;
use crate::wire::{Message, PATH, PROTOCOL, Up};

/// The desk, shared by the runs and the enqueues.
#[derive(Clone)]
pub struct Desk(Arc<Shared>);

struct Shared {
    /// The proxy beside the program: the daemon and the MCP session.
    client: Client,
    /// The connection to the daemon: dialled on first use, kept, and
    /// dialled again when a turn finds it gone.
    handle: Mutex<Option<Handle>>,
    /// The open turn's way up, held weakly: the run holds it, and it
    /// is gone when the run is (rule 2).
    turn: Mutex<Option<mpsc::WeakUnboundedSender<Up>>>,
}

/// Why a run was not started.
pub enum Refused {
    /// A turn is open (rule 2).
    Busy,
    /// The agent is not registered, so it has no desk yet.
    Unregistered,
    /// The desk did not take the turn (rule 1), with why.
    Desk(String),
}

/// Why an enqueue was not taken: no turn is open (rule 4).
pub struct Closed;

/// An open turn, as the run holds it.
pub struct Turn {
    desk: Desk,
    /// The way up. The run's own; a tool call in flight holds another.
    up: mpsc::UnboundedSender<Up>,
    /// What the desk sends down, as the daemon answers it.
    pub answers: api::client::execute::ExecuteStream,
}

impl Desk {
    /// A desk not yet reached.
    pub fn new() -> Self {
        Desk(Arc::new(Shared { client: Client::new(), handle: Mutex::new(None), turn: Mutex::new(None) }))
    }

    /// The connection to the daemon: the one held, or a new one
    /// dialled.
    async fn handle(&self) -> Result<Handle, String> {
        let mut held = self.0.handle.lock().await;
        if let Some(handle) = held.as_ref() {
            return Ok(handle.clone());
        }
        let handle = self.0.client.daemon().await.map_err(|error| error.to_string())?;
        *held = Some(handle.clone());
        Ok(handle)
    }

    /// Rules 1 to 3: a turn opened on the run's messages.
    pub async fn begin(&self, messages: Vec<Message>) -> Result<Turn, Refused> {
        // Held to the end, so two runs cannot both find no turn open.
        let mut turn = self.0.turn.lock().await;
        // Rule 2.
        if turn.as_ref().is_some_and(|open| open.upgrade().is_some()) {
            return Err(Refused::Busy);
        }
        let Some(dependency) = register::desk() else {
            return Err(Refused::Unregistered);
        };
        // Rule 1.
        let handle = self.handle().await.map_err(Refused::Desk)?;
        let request = api::client::request::Frame {
            dependency: dependency.clone(),
            head: Head { method: "GET".to_string(), path: PATH.to_string(), headers: vec![("connection".to_string(), "upgrade".to_string()), ("upgrade".to_string(), PROTOCOL.to_string())] },
            relay: None,
        };
        let (mut answers, way) = match api::client::execute::execute(&handle, &request).await {
            Ok(opened) => opened,
            Err(error) => {
                // The next turn dials again.
                *self.0.handle.lock().await = None;
                return Err(Refused::Desk(error.to_string()));
            }
        };
        match answers.next().await {
            Some(Ok(api::server::response::Frame::Head(head))) if head.status == 101 => {}
            Some(Ok(api::server::response::Frame::Head(head))) => return Err(Refused::Desk(format!("the desk answered {}", head.status))),
            Some(Ok(api::server::response::Frame::NotFound)) => return Err(Refused::Desk("the daemon finds no desk of this agent's".to_string())),
            Some(Ok(api::server::response::Frame::Forbidden)) => return Err(Refused::Desk("the agent may not reach its desk".to_string())),
            Some(Ok(api::server::response::Frame::NoApi)) => return Err(Refused::Desk("the desk's image declares no API".to_string())),
            Some(Ok(api::server::response::Frame::Error(error))) => return Err(Refused::Desk(error.0.to_string())),
            Some(Ok(api::server::response::Frame::Body(_))) => return Err(Refused::Desk("the desk answered a body before a head".to_string())),
            Some(Err(error)) => return Err(Refused::Desk(error.to_string())),
            None => return Err(Refused::Desk("the desk did not answer".to_string())),
        }
        let (up, mut taken) = mpsc::unbounded_channel::<Up>();
        *turn = Some(up.downgrade());
        // Rule 6: every line up, in order, then the stream's end.
        tokio::spawn(async move {
            while let Some(line) = taken.recv().await {
                let Ok(mut text) = serde_json::to_string(&line) else {
                    continue;
                };
                text.push('\n');
                if way.body(text.as_bytes()).await.is_err() {
                    break;
                }
            }
            let _ = way.end().await;
        });
        let turn = Turn { desk: self.clone(), up, answers };
        // Rule 3.
        let _ = turn.up.send(Up::Messages { messages });
        turn.list();
        Ok(turn)
    }

    /// Rule 4.
    pub async fn deliver(&self, messages: Vec<Message>) -> Result<(), Closed> {
        let turn = self.0.turn.lock().await;
        let up = turn.as_ref().and_then(|open| open.upgrade()).ok_or(Closed)?;
        up.send(Up::Messages { messages }).map_err(|_| Closed)
    }
}

impl Turn {
    /// The agent's tools, listed on a task of its own, and sent up:
    /// the listing, or why it could not be had.
    pub fn list(&self) {
        let (desk, up) = (self.desk.clone(), self.up.clone());
        tokio::spawn(async move {
            let listed = match desk.0.client.mcp_peer().await {
                Ok(peer) => peer.list_all_tools().await.map_err(|error| error.to_string()),
                Err(error) => Err(error.to_string()),
            };
            let _ = up.send(match listed {
                Ok(tools) => Up::Tools { tools, error: None },
                Err(error) => Up::Tools { tools: Vec::new(), error: Some(error) },
            });
        });
    }

    /// Rule 5.
    pub fn call(&self, id: String, name: String, arguments: JsonObject) {
        let (desk, up) = (self.desk.clone(), self.up.clone());
        tokio::spawn(async move {
            let mut params = CallToolRequestParams::new(name);
            params.arguments = Some(arguments);
            let result = match desk.0.client.mcp_call_tool(params).await {
                Ok(result) => result,
                Err(error) => CallToolResult::error(vec![ContentBlock::text(format!("the tool could not be called: {error}"))]),
            };
            let _ = up.send(Up::Result { id, result });
        });
    }
}
