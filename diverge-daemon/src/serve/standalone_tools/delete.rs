//! Deleting a tool.

use std::sync::Arc;

use diverge_sdk::daemon::endpoints::standalone_tools::delete::client::request;
use diverge_sdk::daemon::endpoints::standalone_tools::delete::server::response::Frame;
use diverge_sdk::daemon::grant::standalone_tools::Over;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;

use super::{reaches, resolve};
use crate::containers::{ToolKey, eager, tool_log};
use crate::daemon::{Daemon, Kind};
use crate::database;
use crate::judge::{self, Standing, Who};
use crate::serve::reply;
use crate::store::{self, tools};

/// Answer the delete and finish the scope.
pub async fn handle(scope: ScopeHandle, frame: request::Frame, who: Who, daemon: &Arc<Daemon>) {
    let answer = match serve(frame, who, daemon).await {
        Ok(answer) => answer,
        Err(error) => Frame::Error(reply::failure(&error)),
    };
    reply::reply(&scope, &answer).await;
    scope.send_response_finish().await;
}

/// `Forbidden` with no `delete` grant at all; `NotFound`; `Forbidden`
/// for a tool the grants do not reach; `Attached` while it is
/// attached to any agent or any agent template; else the tool gone — its own compartment of the database
/// dropped with its every table, its template's and its image's left, its name free, and, an eager tool, its
/// keeping ended: held by no agent, it may still be held by a connect
/// or be waiting on its clock, and that run is stopped with it. A
/// lazy tool attached nowhere runs nowhere, so nothing is
/// stopped.
async fn serve(frame: request::Frame, who: Who, daemon: &Arc<Daemon>) -> Result<Frame, store::Error> {
    let mut tx = daemon.store.begin().await?;
    let Some(standing) = Standing::of(&mut tx, who).await? else {
        return Ok(Frame::Forbidden);
    };
    if !judge::standalone_tools::holds(&standing, Over::Delete) {
        return Ok(Frame::Forbidden);
    }
    let Some(tool) = resolve(&mut tx, &frame.tool, true).await? else {
        return Ok(Frame::NotFound);
    };
    if !reaches(&mut tx, daemon, &standing, Over::Delete, &tool).await? {
        return Ok(Frame::Forbidden);
    }
    let attached = !tools::attachments::of_tool(&mut tx, tool.id).await?.is_empty() || !tools::template_attachments::of_tool(&mut tx, tool.id).await?.is_empty();
    if attached {
        return Ok(Frame::Attached);
    }
    tools::delete(&mut tx, tool.id).await?;
    database::remove(daemon, &mut tx, &database::instance_of_tool(&tool)).await?;
    tx.commit().await?;
    eager::tool_deleted(daemon, tool.id).await;
    tool_log::forget(daemon, ToolKey::Record(tool.id)).await;
    daemon.live.changed(Kind::StandaloneTools);
    daemon.live.changed(Kind::Volumes);
    daemon.live.changed(Kind::Vault);
    daemon.live.changed(Kind::PostgresCompartments);
    Ok(Frame::Deleted)
}
