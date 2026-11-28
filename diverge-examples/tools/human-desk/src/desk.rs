//! The desk: where the agent's turn and the person meet.
//!
//! One per container, shared by the agent's stream and every open
//! page. It holds the transcript, the open turn, if any, the agent's
//! tools as last listed, and the fan-out that tells every page what
//! happens.
//!
//! The rules:
//!
//! 1. A turn opens when the agent opens its stream, and at most one
//!    is open. A stream that comes while one is open is refused,
//!    unless the open one's stream is gone, which closes it.
//! 2. Every chunk is recorded the same way: given the next place in
//!    the transcript, kept in the database, sent down the open turn's
//!    stream, and told to every page. A chunk recorded with no turn
//!    open is kept and told, and goes down no stream.
//! 3. The person's acts — a reply, a tool call, the turn's end — need
//!    an open turn; with none, each is refused, with why.
//! 4. A reply is one `assistant_text_content` chunk.
//! 5. A tool call is an `assistant_tool_call` chunk under an id minted
//!    here, its arguments as JSON text, and then the call itself, sent
//!    down the stream for the agent to make: the tools are the
//!    agent's. Its result comes back up under the same id and is
//!    recorded as the `tool_response`.
//! 6. The turn's end is sent down the stream, and the turn is closed.
//!    It is refused while a tool call is in flight, so no response is
//!    left off the run.
//! 7. Messages that come up the stream while the turn is open are
//!    taken at once: every part is recorded under its own message's
//!    key.
//! 8. The agent's tools are kept as the agent last listed them, and
//!    told to every page when a listing comes. Asked for again, they
//!    are asked of the agent where a turn is open, and are the last
//!    listing where none is.
//! 9. The transcript is read from the database the first time the desk
//!    is used, and kept in memory after.

use std::sync::Arc;

use diverge_sdk::container_proxy::inside::Client;
use diverge_sdk::provider::endpoints::containers::agents::run::server::response::{
    AgenticLoopChunk, AssistantTextContentChunk, AssistantToolCallChunk, ToolResponseChunk, user_parts,
};
use rmcp::model::{CallToolResult, JsonObject, Tool};
use serde::Serialize;
use tokio::sync::{Mutex, MutexGuard, broadcast, mpsc};

use crate::store::{Entry, Store};
use crate::wire::{Down, Message};

/// What a page is told.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Told {
    /// A chunk recorded.
    Entry { entry: Entry },
    /// Whether a turn is open: the agent waiting on the person.
    State { waiting: bool },
    /// The agent's tools, as it last listed them (rule 8).
    Tools {
        tools: Vec<Tool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    /// Something went wrong that is no one act's: a chunk that could
    /// not be kept.
    Trouble { error: String },
}

/// The desk, shared.
#[derive(Clone)]
pub struct Desk(Arc<Shared>);

struct Shared {
    inner: Mutex<Inner>,
    told: broadcast::Sender<Told>,
    client: Client,
    store: Store,
}

#[derive(Default)]
struct Inner {
    /// Whether the transcript has been read (rule 9).
    loaded: bool,
    transcript: Vec<Entry>,
    /// The open turn's stream, downward.
    turn: Option<mpsc::UnboundedSender<Down>>,
    /// The ids minted so far, for the next.
    calls: u64,
    /// The tool calls in flight.
    pending: usize,
    /// The agent's tools, as last listed, and why not where the
    /// listing failed (rule 8). `None` before any listing.
    tools: Option<(Vec<Tool>, Option<String>)>,
}

/// Why a stream was not taken as a turn: one is open (rule 1).
pub struct Busy;

impl Desk {
    /// A desk with nothing on it.
    pub fn new() -> Self {
        let (told, _) = broadcast::channel(256);
        Desk(Arc::new(Shared { inner: Mutex::new(Inner::default()), told, client: Client::new(), store: Store::default() }))
    }

    /// Listen for what pages are told.
    pub fn listen(&self) -> broadcast::Receiver<Told> {
        self.0.told.subscribe()
    }

    /// The lock, the transcript read first if it has not been (rule 9).
    async fn lock(&self) -> MutexGuard<'_, Inner> {
        let mut inner = self.0.inner.lock().await;
        if !inner.loaded {
            match self.0.store.load(&self.0.client).await {
                Ok(transcript) => {
                    inner.transcript = transcript;
                    inner.loaded = true;
                }
                Err(error) => {
                    let _ = self.0.told.send(Told::Trouble { error: format!("the transcript could not be read: {error}") });
                }
            }
        }
        inner
    }

    /// Whether a turn is open, closing one whose stream is gone
    /// (rule 1).
    fn open(&self, inner: &mut Inner) -> bool {
        if inner.turn.as_ref().is_some_and(|turn| turn.is_closed()) {
            inner.turn = None;
            inner.pending = 0;
            let _ = self.0.told.send(Told::State { waiting: false });
        }
        inner.turn.is_some()
    }

    /// Rule 2.
    async fn record(&self, inner: &mut Inner, chunk: AgenticLoopChunk) {
        let entry = Entry { id: inner.transcript.last().map_or(0, |last| last.id + 1), chunk };
        if let Err(error) = self.0.store.append(&self.0.client, &entry).await {
            let _ = self.0.told.send(Told::Trouble { error: format!("a chunk could not be kept: {error}") });
        }
        if let Some(turn) = &inner.turn {
            let _ = turn.send(Down::Chunk { chunk: entry.chunk.clone() });
        }
        inner.transcript.push(entry.clone());
        let _ = self.0.told.send(Told::Entry { entry });
    }

    /// The parts of `messages`, recorded in order.
    async fn take(&self, inner: &mut Inner, messages: Vec<Message>) {
        for message in messages {
            for chunk in user_parts(&message.key, message.content) {
                self.record(inner, chunk).await;
            }
        }
    }

    /// The transcript and whether a turn is open, as a page first sees
    /// them.
    pub async fn snapshot(&self) -> (Vec<Entry>, bool) {
        let mut inner = self.lock().await;
        let waiting = self.open(&mut inner);
        (inner.transcript.clone(), waiting)
    }

    /// Rule 1: a turn opened on the stream's first messages, their
    /// parts recorded first; what goes down the stream, until the turn
    /// ends.
    pub async fn begin(&self, messages: Vec<Message>) -> Result<mpsc::UnboundedReceiver<Down>, Busy> {
        let mut inner = self.lock().await;
        if self.open(&mut inner) {
            return Err(Busy);
        }
        let (turn, stream) = mpsc::unbounded_channel();
        inner.turn = Some(turn);
        inner.pending = 0;
        self.take(&mut inner, messages).await;
        let _ = self.0.told.send(Told::State { waiting: true });
        Ok(stream)
    }

    /// Rule 7. Messages that come with no turn open are dropped: the
    /// stream they came on is not the turn's.
    pub async fn deliver(&self, messages: Vec<Message>) {
        let mut inner = self.lock().await;
        if self.open(&mut inner) {
            self.take(&mut inner, messages).await;
        }
    }

    /// The stream is gone: the turn it was, if it is still the open
    /// one, is closed, and every page told.
    pub async fn gone(&self) {
        let mut inner = self.lock().await;
        self.open(&mut inner);
    }

    /// Rule 8: a listing, kept and told.
    pub async fn listed(&self, tools: Vec<Tool>, error: Option<String>) {
        let mut inner = self.lock().await;
        inner.tools = Some((tools.clone(), error.clone()));
        let _ = self.0.told.send(Told::Tools { tools, error });
    }

    /// Rule 8: the tools, for a page. With a turn open the agent is
    /// asked again, and its answer is told to every page; the last
    /// listing is what the asking page is handed meanwhile.
    pub async fn tools(&self, again: bool) -> Told {
        let mut inner = self.lock().await;
        if again && self.open(&mut inner) {
            if let Some(turn) = &inner.turn {
                let _ = turn.send(Down::Tools);
            }
        }
        match &inner.tools {
            Some((tools, error)) => Told::Tools { tools: tools.clone(), error: error.clone() },
            None => Told::Tools { tools: Vec::new(), error: Some("the agent lists its tools when a message is waiting on you".to_string()) },
        }
    }

    /// Rule 4.
    pub async fn reply(&self, text: String) -> Result<(), String> {
        let mut inner = self.lock().await;
        if !self.open(&mut inner) {
            return Err("no message is waiting on you".to_string());
        }
        let chunk = AssistantTextContentChunk { r#type: Default::default(), parent_tool_call_id: None, logprobs: None, inner: rmcp::model::TextContent::new(text) };
        self.record(&mut inner, AgenticLoopChunk::AssistantTextContent(chunk)).await;
        Ok(())
    }

    /// Rule 5, its first half: the call recorded and sent down for the
    /// agent to make, or refused.
    pub async fn place(&self, name: String, arguments: JsonObject) -> Result<(), String> {
        let mut inner = self.lock().await;
        if !self.open(&mut inner) {
            return Err("no message is waiting on you".to_string());
        }
        let text = serde_json::to_string(&arguments).map_err(|error| error.to_string())?;
        inner.calls += 1;
        inner.pending += 1;
        let id = format!("call_{}", inner.calls);
        let chunk = AssistantToolCallChunk { r#type: Default::default(), parent_tool_call_id: None, id: id.clone(), meta: None, name: name.clone(), arguments: Some(text) };
        self.record(&mut inner, AgenticLoopChunk::AssistantToolCall(chunk)).await;
        if let Some(turn) = &inner.turn {
            let _ = turn.send(Down::Call { id, name, arguments });
        }
        Ok(())
    }

    /// Rule 5, its second half: the agent's result, recorded under the
    /// call's id. A result that comes with no turn open is dropped.
    pub async fn result(&self, id: String, result: CallToolResult) {
        let mut inner = self.lock().await;
        if !self.open(&mut inner) {
            return;
        }
        inner.pending = inner.pending.saturating_sub(1);
        let chunk = ToolResponseChunk { r#type: Default::default(), parent_tool_call_id: None, id, inner: result };
        self.record(&mut inner, AgenticLoopChunk::ToolResponse(chunk)).await;
    }

    /// Rule 6.
    pub async fn end(&self) -> Result<(), String> {
        let mut inner = self.lock().await;
        if !self.open(&mut inner) {
            return Err("no message is waiting on you".to_string());
        }
        if inner.pending > 0 {
            return Err("a tool call is still running".to_string());
        }
        if let Some(turn) = inner.turn.take() {
            let _ = turn.send(Down::End);
        }
        let _ = self.0.told.send(Told::State { waiting: false });
        Ok(())
    }
}
