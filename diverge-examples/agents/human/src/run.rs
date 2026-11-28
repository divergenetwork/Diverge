//! `POST /run` and `POST /enqueue`: the loop, which is the person at
//! the agent's desk.
//!
//! The rules:
//!
//! 1. A run before the registration is `409`, `unregistered`; a run
//!    with no message, or a message with no content, is `400`; a run
//!    while a turn is open is `409`, `busy`; a run whose desk does not
//!    take the turn is `502`, `desk`, with why.
//! 2. Otherwise the run is `200`, server-sent events: every chunk the
//!    desk records on the turn, each one event's data, in the desk's
//!    order — the messages' parts first, then whatever the person
//!    does — until the desk says the turn has ended, which ends the
//!    stream. Nothing times it out: the agent waits on the person.
//! 3. A tool call the desk asks for is made by the agent, and the
//!    tools are listed by it, since both are the agent's: see
//!    [`Turn`](crate::desk::Turn).
//! 4. A desk that stops answering, or a line of its that does not
//!    read, ends the run as the turn's end does.
//! 5. A comment goes out on the stream when it is quiet, so the
//!    connection is never idle for long; the proxy reads only data.
//! 6. An enqueue with no message, or a message with no content, is
//!    `400`, checked before anything is accepted; otherwise its
//!    messages — the proxy's joined batch — go to the desk on the
//!    open turn at once and are answered `204`, their delivery, with
//!    nothing waiting on the person; with no turn open the answer is
//!    `409`, `closed`, and the proxy starts the next run on them.

use std::convert::Infallible;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use diverge_sdk::container_proxy::inside::agent::enqueue;
use diverge_sdk::container_proxy::inside::agent::run;
use diverge_sdk::daemon::endpoints::dependency_tools::api::server::response::Frame;
use futures_util::{Stream, StreamExt as _};
use serde_json::{Value, json};

use crate::desk::{Closed, Desk, Refused};
use crate::wire::{Down, Message};

/// A refusal: the status, and a JSON reason.
type Refusal = (StatusCode, Json<Value>);

/// The messages of a run or an enqueue, as the desk takes them.
fn messages(request: enqueue::request::Request) -> Vec<Message> {
    request.messages.into_iter().map(|message| Message { key: message.key, content: message.content }).collect()
}

/// Rules 1 to 5.
pub async fn run(State(desk): State<Desk>, Json(request): Json<run::request::Request>) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Refusal> {
    // The run is a chat run, or it is refused: this agent's image
    // declares no structured decisions, so a decision is never taken.
    // Refused before anything is claimed, so nothing is to undo.
    let request = match request {
        run::request::Request::Chat { messages } => enqueue::request::Request { messages },
        run::request::Request::StructuredDecisions(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "kind": "content",
                    "error": "structured decisions are not supported by this agent",
                })),
            ));
        }
    };
    if request.messages.is_empty() || request.messages.iter().any(|message| message.content.is_empty()) {
        return Err((StatusCode::BAD_REQUEST, Json(json!({ "kind": "content", "error": "a run needs messages, each with content" }))));
    }
    let mut turn = match desk.begin(messages(request)).await {
        Ok(turn) => turn,
        Err(Refused::Unregistered) => return Err((StatusCode::CONFLICT, Json(json!({ "kind": "unregistered", "error": "the agent is not registered" })))),
        Err(Refused::Busy) => return Err((StatusCode::CONFLICT, Json(json!({ "kind": "busy", "error": "a turn is open" })))),
        Err(Refused::Desk(why)) => return Err((StatusCode::BAD_GATEWAY, Json(json!({ "kind": "desk", "error": why })))),
    };
    let events = async_stream::stream! {
        // What has come down and is not yet a whole line.
        let mut buffer: Vec<u8> = Vec::new();
        'turn: while let Some(Ok(Frame::Body(bytes))) = turn.answers.next().await {
            buffer.extend_from_slice(&bytes);
            while let Some(at) = buffer.iter().position(|byte| *byte == b'\n') {
                let line: Vec<u8> = buffer.drain(..=at).collect();
                match serde_json::from_slice::<Down>(&line[..at]) {
                    Ok(Down::Chunk { chunk }) => {
                        if let Ok(data) = serde_json::to_string(&chunk) {
                            yield Ok(Event::default().data(data));
                        }
                    }
                    // Rule 3.
                    Ok(Down::Call { id, name, arguments }) => turn.call(id, name, arguments),
                    Ok(Down::Tools) => turn.list(),
                    // Rules 2 and 4.
                    Ok(Down::End) | Err(_) => break 'turn,
                }
            }
        }
    };
    Ok(Sse::new(events).keep_alive(KeepAlive::default()))
}

/// Rule 6.
pub async fn enqueue(State(desk): State<Desk>, Json(request): Json<enqueue::request::Request>) -> Result<StatusCode, Refusal> {
    if request.messages.is_empty() || request.messages.iter().any(|message| message.content.is_empty()) {
        return Err((StatusCode::BAD_REQUEST, Json(json!({ "kind": "content", "error": "an enqueue needs messages, each with content" }))));
    }
    match desk.deliver(messages(request)).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(Closed) => Err((StatusCode::CONFLICT, Json(json!({ "kind": "closed", "error": "no turn is open" })))),
    }
}
