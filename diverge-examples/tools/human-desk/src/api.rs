//! The API, beneath `/api`: the agent's stream, one per turn.
//!
//! The rules, with [`wire`](crate::wire)'s:
//!
//! 1. `GET` [`PATH`] from the agent the desk is deployed for is
//!    answered `101`, and the connection is the stream from then. A
//!    request with no stamp, one stamped as a client or as a tool, or
//!    one stamped as any other agent, is `403`; one that came before
//!    the desk's registration is `409`.
//! 2. The first line up is the run's messages, which opens the turn.
//!    A first line that is anything else, or a turn already open, ends
//!    the stream with nothing sent.
//! 3. After that each line up is taken as it comes, and everything
//!    the desk has for the agent goes down, in order, until the
//!    turn's end has gone down — the desk then closes its side — or
//!    the agent closes its own, which is the turn gone.
//! 4. A line that does not read ends the stream.

use axum::Router;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use hyper_util::rt::TokioIo;
use serde_json::json;
use tokio::io::{AsyncBufReadExt as _, AsyncRead, AsyncWrite, AsyncWriteExt as _, BufReader};

use crate::caller::agent;
use crate::desk::Desk;
use crate::register::owner;
use crate::wire::{Down, PATH, PROTOCOL, Up};

/// The routes beneath `/api`, holding the desk.
pub fn routes(desk: Desk) -> Router {
    Router::new().route(PATH, get(open)).with_state(desk)
}

/// A refusal: the status, and why.
fn refused(status: StatusCode, why: &str) -> Response {
    (status, axum::Json(json!({ "error": why }))).into_response()
}

/// Rule 1.
async fn open(State(desk): State<Desk>, mut request: Request) -> Response {
    let Some(owner) = owner() else {
        return refused(StatusCode::CONFLICT, "the desk is not registered");
    };
    match agent(request.headers()) {
        Ok(caller) if caller.template == owner.template && caller.index == owner.index => {}
        Ok(_) => return refused(StatusCode::FORBIDDEN, "another agent's desk"),
        Err(why) => return refused(StatusCode::FORBIDDEN, &why),
    }
    let upgrade = hyper::upgrade::on(&mut request);
    tokio::spawn(async move {
        if let Ok(upgraded) = upgrade.await {
            sit(TokioIo::new(upgraded), desk).await;
        }
    });
    match Response::builder().status(StatusCode::SWITCHING_PROTOCOLS).header(header::CONNECTION, "upgrade").header(header::UPGRADE, PROTOCOL).body(Body::empty()) {
        Ok(response) => response,
        Err(error) => refused(StatusCode::INTERNAL_SERVER_ERROR, &error.to_string()),
    }
}

/// Rules 2 to 4: the agent at the desk, for one turn.
async fn sit<S: AsyncRead + AsyncWrite + Unpin>(stream: S, desk: Desk) {
    let (read, mut write) = tokio::io::split(stream);
    let mut lines = BufReader::new(read).lines();
    // Rule 2.
    let first = match lines.next_line().await {
        Ok(Some(line)) => serde_json::from_str::<Up>(&line),
        _ => return,
    };
    let Ok(Up::Messages { messages }) = first else {
        return;
    };
    let Ok(mut down) = desk.begin(messages).await else {
        return;
    };
    // Rule 3.
    loop {
        tokio::select! {
            next = down.recv() => {
                let Some(next) = next else {
                    break;
                };
                let ended = matches!(next, Down::End);
                if !send(&mut write, &next).await || ended {
                    break;
                }
            }
            line = lines.next_line() => {
                let Ok(Some(line)) = line else {
                    break;
                };
                match serde_json::from_str::<Up>(&line) {
                    Ok(Up::Messages { messages }) => desk.deliver(messages).await,
                    Ok(Up::Tools { tools, error }) => desk.listed(tools, error).await,
                    Ok(Up::Result { id, result }) => desk.result(id, result).await,
                    // Rule 4.
                    Err(_) => break,
                }
            }
        }
    }
    let _ = write.shutdown().await;
    // The receiver goes first, so the desk sees the stream gone.
    drop(down);
    desk.gone().await;
}

/// One line down; whether it went.
async fn send<W: AsyncWrite + Unpin>(write: &mut W, line: &Down) -> bool {
    let Ok(mut text) = serde_json::to_string(line) else {
        return true;
    };
    text.push('\n');
    write.write_all(text.as_bytes()).await.is_ok() && write.flush().await.is_ok()
}
