//! An agent container's begin: the arguments registered, the queue's
//! driver started, the family's channels served.

use std::sync::Arc;

use diverge_sdk::container_proxy::outside::endpoints::agents::begin::client::{channel_request, request};
use diverge_sdk::container_proxy::outside::endpoints::agents::begin::server::response;
use diverge_sdk::wire::decode::Decode as _;
use diverge_sdk::wire::frame::client::ClientFrame;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;
use diverge_sdk::shared::error::Error;
use tokio::task::JoinSet;

use super::Family;
use crate::encode::encoded;
use crate::proxy::{Begun, Proxy};
use crate::{agent, inside, program};

/// Serve the begin for the connection's life.
///
/// A second begin on the connection is `Error`, then the finish, and
/// the first goes on. Otherwise the arguments, the template and the
/// image are registered with the program's server — its refusal is the `Error`,
/// in its own words, and the connection has not begun — and then
/// `Begun` goes out carrying the dependencies and the permissions
/// the program answered with.
/// The scope is published to
/// every surface inside the container, the queue's driver and the
/// resident notifications ask are started, and every channel the
/// server opens on the scope is served on a task of its own; a
/// channel this end cannot read is finished with nothing. The
/// server's half of each daemon connection the program opened is
/// among them. The
/// scope is the connection's life.
///
/// Every task it starts is its own — the channels', and for an agent
/// the resident notifications ask — held in one set: those that ended
/// are reaped before the next channel is taken, so the set holds what
/// is in flight and no more; the finish is followed by the rest,
/// awaited, each ending with its channel; and the connection ending,
/// which aborts this task, aborts every one of them with it.
pub async fn agents(proxy: Arc<Proxy>, scope: ScopeHandle, frame: request::Frame) {
    if !proxy.claim_begin().await {
        refuse(&scope, begun()).await;
        return;
    }
    let registered = match program::register(&proxy.upstream, frame.arguments, frame.template, frame.image).await {
        Ok(registered) => registered,
        Err(error) => {
            proxy.release_begin().await;
            refuse(&scope, error).await;
            return;
        }
    };
    let scope = Arc::new(scope);
    let begun = response::Registered {
        dependencies: registered.dependencies,
        permissions: registered.permissions,
    };
    if let Some(payload) = encoded(&response::Frame::Begun(begun)) {
        scope.send_response(&payload).await;
    }
    proxy.publish(Begun {
        scope: Arc::clone(&scope),
        family: Family::Agents,
    });
    proxy.set_commands(agent::driver(Arc::clone(&proxy), Arc::clone(&scope)));
    let mut tasks = JoinSet::new();
    tasks.spawn(inside::mcp::notifications(Arc::clone(&proxy)));

    loop {
        // Finished ones, so the set does not grow for the life of the
        // connection.
        while tasks.try_join_next().is_some() {}
        let Some(bytes) = scope.recv_channel_request().await else {
            break;
        };
        let Ok(ClientFrame::ChannelRequest { channel, payload, .. }) = ClientFrame::decode(&bytes) else {
            continue;
        };
        let proxy = Arc::clone(&proxy);
        let scope = Arc::clone(&scope);
        match channel_request::Frame::decode(payload) {
            Ok(channel_request::Frame::Postgres(request)) => {
                tasks.spawn(inside::postgres::attach(proxy, scope, channel, request.connection_id));
            }
            Ok(channel_request::Frame::Enqueue(request)) => {
                tasks.spawn(agent::enqueue(proxy, scope, channel, request));
            }
            Ok(channel_request::Frame::Daemon(request)) => {
                tasks.spawn(inside::daemon::attach(proxy, scope, channel, request.connection_id));
            }
            Err(_) => {
                tasks.spawn(async move { scope.send_channel_response_finish(channel).await });
            }
        }
    }
    scope.send_response_finish().await;
    while tasks.join_next().await.is_some() {}
}

/// The `Error`, then the finish: the connection has not begun.
async fn refuse(scope: &ScopeHandle, error: Error) {
    if let Some(payload) = encoded(&response::Frame::Error(error)) {
        scope.send_response(&payload).await;
    }
    scope.send_response_finish().await;
}

/// A second begin on a connection that had one.
pub(super) fn begun() -> Error {
    Error(serde_json::json!({
        "kind": "begun",
        "error": "this connection has begun",
    }))
}
