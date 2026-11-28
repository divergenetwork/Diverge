//! The wire between the two halves of the human example: the `human`
//! agent and its `human-desk` tool.
//!
//! This file is kept twice, word for word: as
//! `diverge-examples/agents/human/src/wire.rs` and as
//! `diverge-examples/tools/human-desk/src/wire.rs`. A change to one is
//! a change to the other.
//!
//! The agent opens one request on the desk's API for each run —
//! `GET` [`PATH`], with `Connection: upgrade` and `Upgrade:`
//! [`PROTOCOL`] — and the desk answers `101`. From then the two speak
//! lines: each one JSON value and a newline, an [`Up`] from the agent
//! and a [`Down`] from the desk. The rules:
//!
//! 1. The stream is the turn: it opens with the run, and the first
//!    line on it is an [`Up::Messages`], the run's own.
//! 2. Every chunk of the turn is the desk's to record, and comes down
//!    as a [`Down::Chunk`], in order: the messages' parts, the
//!    person's replies, each tool call and each tool's response. The
//!    agent's run is those chunks and nothing else.
//! 3. A tool call is made by the agent, which holds the tools: the
//!    desk asks with a [`Down::Call`], after the call's own chunk, and
//!    the agent answers with an [`Up::Result`] under the same id,
//!    whatever came of it.
//! 4. The tools are listed by the agent: once unasked, after its first
//!    line, and again at every [`Down::Tools`].
//! 5. The turn ends with a [`Down::End`], after which the desk closes
//!    its side. Either side closing without one is the turn gone.

use diverge_sdk::provider::endpoints::containers::agents::run::server::response::AgenticLoopChunk;
use rmcp::model::{CallToolResult, ContentBlock, JsonObject, Tool};
use serde::{Deserialize, Serialize};

/// The path of the desk's API the stream is opened on.
pub const PATH: &str = "/api/desk";

/// What the request upgrades to, as its `Upgrade` header names it.
pub const PROTOCOL: &str = "human-desk";

/// One message for the agent: the key the caller sent it under, and
/// its content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The message's key.
    pub key: String,
    /// Its content, block by block.
    pub content: Vec<ContentBlock>,
}

/// A line from the agent to the desk.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Up {
    /// Messages for the turn: the run's own, first, and every batch
    /// enqueued while the turn is open (rule 1).
    Messages {
        /// The messages, in order.
        messages: Vec<Message>,
    },
    /// The agent's tools, listed, or why they could not be (rule 4).
    Tools {
        /// The tools; empty where there are none, or on an error.
        tools: Vec<Tool>,
        /// Why the listing failed, where it did.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    /// What a tool call came to (rule 3).
    Result {
        /// The call's id, as the desk minted it.
        id: String,
        /// The tool's result; a call that failed as a link is an error
        /// result.
        result: CallToolResult,
    },
}

/// A line from the desk to the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Down {
    /// A chunk recorded on the turn (rule 2).
    Chunk {
        /// The chunk, as the run emits it.
        chunk: AgenticLoopChunk,
    },
    /// A tool call for the agent to make (rule 3).
    Call {
        /// The call's id.
        id: String,
        /// The tool's name, as the agent's listing shows it.
        name: String,
        /// The tool's arguments.
        arguments: JsonObject,
    },
    /// List the tools again (rule 4).
    Tools,
    /// The person ended the turn (rule 5).
    End,
}
