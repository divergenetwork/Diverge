//! One HTTP request through one of a standalone tool's two doors: its
//! frontend, or its API.

use std::sync::Arc;

use diverge_sdk::daemon::grant::standalone_tools::Over;
use diverge_sdk::daemon::reference;
use diverge_sdk::shared::containers::frontend::request::Head;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;

use super::{reaches, resolve};
use crate::containers::{self, Door};
use crate::daemon::Daemon;
use crate::judge::{self, Standing, Who};
use crate::serve::{files, reply};
use crate::store::{self, tools_templates};

/// The action a request through `door` takes over the tool.
fn action(door: Door) -> Over {
    match door {
        Door::Frontend => Over::Frontend,
        Door::Api => Over::Api,
    }
}

/// Send the request through `door`, serve its answer and its body,
/// and finish the scope: a [`frontend`](super::frontend) request and
/// an [`api`](super::api) request alike, each answered in its own
/// frame `A`.
pub async fn handle<A: files::Answer>(scope: ScopeHandle, tool: reference::Tool, head: Head, who: Who, daemon: &Arc<Daemon>, door: Door) {
    if let Err(error) = serve::<A>(&scope, tool, head, who, daemon, door).await {
        reply::reply(&scope, &A::error(reply::failure(&error))).await;
    }
    scope.send_response_finish().await;
}

/// The rules, in the order they are answered:
///
/// 1. `Forbidden` with no grant of the door's action at all — whoever
///    asks: an account, an agent's container or a tool's is judged
///    alike.
/// 2. `NotFound` for a tool that is none.
/// 3. `Forbidden` for a tool the grants do not reach.
/// 4. An `Error` for a requester that cannot be attested as a caller
///    (see [`files::caller`]).
/// 5. The door's `NoFrontend` or `NoApi` for a tool whose image
///    declares none, before anything is started.
/// 6. The container started if it was down, which failing is the
///    `Error` — a connected tool's doors are the other daemon's, and
///    its is the `Error` too.
/// 7. The request (see [`files::door`]), the container held up while
///    it is answered.
async fn serve<A: files::Answer>(scope: &ScopeHandle, tool: reference::Tool, head: Head, who: Who, daemon: &Arc<Daemon>, door: Door) -> Result<(), store::Error> {
    let mut conn = daemon.store.acquire().await?;
    let Some(standing) = Standing::of(&mut conn, who.clone()).await? else {
        reply::reply(scope, &A::forbidden()).await;
        return Ok(());
    };
    // Rule 1.
    if !judge::standalone_tools::holds(&standing, action(door)) {
        reply::reply(scope, &A::forbidden()).await;
        return Ok(());
    }
    // Rule 2.
    let Some(tool) = resolve(&mut conn, &tool, false).await? else {
        reply::reply(scope, &A::not_found()).await;
        return Ok(());
    };
    // Rule 3.
    let reached = reaches(&mut conn, daemon, &standing, action(door), &tool).await?;
    if !reached {
        reply::reply(scope, &A::forbidden()).await;
        return Ok(());
    }
    let image = tools_templates::by_id(&mut conn, &tool.template, false).await?.map(|record| containers::image(&record.template.image));
    drop(conn);
    scope.allowed().await;
    // Rule 4.
    let caller = match files::caller(daemon, &who, &standing).await {
        Ok(caller) => caller,
        Err(error) => {
            reply::reply(scope, &A::error(reply::failure(&error))).await;
            return Ok(());
        }
    };
    let Some(image) = image else {
        reply::reply(scope, &A::error(reply::failure(&"the tool's template is not on record"))).await;
        return Ok(());
    };
    // Rule 5.
    match files::declares(daemon, &image, door).await? {
        Ok(true) => {}
        Ok(false) => {
            reply::reply(scope, &A::undeclared()).await;
            return Ok(());
        }
        Err(error) => {
            reply::reply(scope, &A::error(reply::failure(&error))).await;
            return Ok(());
        }
    }
    // Rule 6.
    let opened = match files::open_tool(daemon, &tool, door.cause()).await {
        Ok(opened) => opened,
        Err(error) => {
            reply::reply(scope, &A::error(reply::failure(&error))).await;
            return Ok(());
        }
    };
    opened.touch();
    // Rule 7.
    if let Err(error) = files::door::<A>(scope, &opened, head, &caller, door).await {
        reply::reply(scope, &A::error(reply::failure(&error))).await;
    }
    Ok(())
}
