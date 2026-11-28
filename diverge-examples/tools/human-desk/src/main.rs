//! `human-desk`, as a tool container: the desk a person answers at as
//! the `human` agent it is deployed for.
//!
//! The agent has no model and no page. When a run comes to it, it
//! opens a stream to this desk and hands the run's messages over;
//! nothing answers them until the person does. The frontend shows
//! them, and the person writes the agent's replies, calls the agent's
//! tools by hand — the desk asks the agent to make each call, since
//! the tools are the agent's — and ends the turn, which ends the run.
//!
//! The program is an HTTP server on the container's loopback, at the
//! port [`diverge_sdk::container_proxy::inside::port()`] names, and the
//! proxy beside it is its only caller: `POST /register` once, the MCP
//! server at `/mcp`, which serves no tool ([`mcp`]), the frontend
//! beneath `/frontend` ([`frontend`]), and the agent's stream beneath
//! `/api` ([`api`], whose lines are [`wire`]'s). `--schema`, the
//! program's one argument, prints the JSON Schema of the arguments it
//! takes, which is the `schema` of the image's `/.diverge/DIVERGE.md`
//! frontmatter.
//!
//! # Nothing of the proxy's before it is needed
//!
//! The host injects the proxy at runtime, and it may not be up until a
//! request comes. So nothing here touches it at registration: the
//! database is reached on first use, and kept.

mod api;
mod arguments;
mod caller;
mod desk;
mod frontend;
mod mcp;
mod register;
mod store;
mod wire;

use std::io::Write as _;
use std::sync::Arc;

use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::{StreamableHttpServerConfig, StreamableHttpService};

use crate::arguments::Arguments;
use crate::desk::Desk;

/// The program: the server, or — on `--schema` — the schema.
///
/// `--schema` alone prints the JSON Schema of [`Arguments`] on stdout
/// and exits 0; no argument is the server, for the container's life;
/// anything else is a usage line on stderr and exit 2.
fn main() {
    let mut arguments = std::env::args_os().skip(1);
    match (arguments.next(), arguments.next()) {
        (None, None) => {}
        (Some(argument), None) if argument == "--schema" => return schema(),
        _ => {
            let _ = writeln!(std::io::stderr(), "usage: human-desk [--schema]");
            std::process::exit(2);
        }
    }
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("the runtime could not be built");
    runtime.block_on(serve());
}

/// `--schema`: the JSON Schema of [`Arguments`], pretty-printed on
/// stdout with a trailing newline.
fn schema() {
    let text = match serde_json::to_string_pretty(&schemars::schema_for!(Arguments)) {
        Ok(text) => text,
        Err(error) => {
            let _ = writeln!(std::io::stderr(), "{error}");
            std::process::exit(1);
        }
    };
    if let Err(error) = writeln!(std::io::stdout(), "{text}") {
        let _ = writeln!(std::io::stderr(), "{error}");
        std::process::exit(1);
    }
}

/// The server, for the container's life, bound on the loopback. The
/// agent's stream and every open page share one [`Desk`].
async fn serve() {
    let desk = Desk::new();
    let served = StreamableHttpService::new(|| Ok(mcp::Nothing), Arc::new(LocalSessionManager::default()), StreamableHttpServerConfig::default());
    let app = axum::Router::new()
        .route("/register", axum::routing::post(register::register))
        .nest_service("/mcp", served)
        .merge(frontend::routes(desk.clone()))
        .merge(api::routes(desk));
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", diverge_sdk::container_proxy::inside::port())).await.expect("the port could not be bound");
    axum::serve(listener, app).await.expect("the server stopped unexpectedly");
}
