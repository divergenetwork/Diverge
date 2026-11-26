//! Deleting a compartment.

use std::sync::Arc;

use diverge_sdk::daemon::endpoints::postgres::compartments::delete::client::request;
use diverge_sdk::daemon::endpoints::postgres::compartments::delete::server::response::Frame;
use diverge_sdk::daemon::endpoints::postgres::compartments::{Compartment, Kind as Of};
use diverge_sdk::daemon::grant::postgres_compartments::Over;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;

use super::{deletable, find};
use crate::containers::{ToolKey, eager, tool_log};
use crate::daemon::{Daemon, Kind};
use crate::database;
use crate::judge::filter::postgres_compartments::Facts;
use crate::judge::{self, Standing, Who};
use crate::serve::dependency_tools::load;
use crate::serve::reply;
use crate::store::{self, agents, dependencies, dependency_tools};

/// Answer the delete and finish the scope.
pub async fn handle(scope: ScopeHandle, frame: request::Frame, who: Who, daemon: &Arc<Daemon>) {
    let answer = match serve(frame, who, daemon).await {
        Ok(answer) => answer,
        Err(error) => Frame::Error(reply::failure(&error)),
    };
    reply::reply(&scope, &answer).await;
    scope.send_response_finish().await;
}

/// The rules, in order:
///
/// 1. `Forbidden` with no `delete` grant at all.
/// 2. `NotFound` for a compartment that is on no record.
/// 3. `Forbidden` for one the grants do not reach.
/// 4. `NotDeletable` for one a delete would not be allowed for now:
///    see [`Holds`](crate::store::postgres_compartments::Holds).
/// 5. `InUse` while a container holds a connection through it, or —
///    for a dependency template's — while any tool of the template
///    runs, lazy or eager.
/// 6. Else the compartment gone: its role, its schema and every table
///    in it, and its record. A dependency template's takes with it
///    every tool of the template — each eager tool among them
///    retired, each one's own `instance` compartment removed — and
///    the template's body, so that a registration declaring it anew
///    makes it anew, with new compartments.
async fn serve(frame: request::Frame, who: Who, daemon: &Arc<Daemon>) -> Result<Frame, store::Error> {
    let mut tx = daemon.store.begin().await?;
    // Rule 1.
    let Some(standing) = Standing::of(&mut tx, who).await? else {
        return Ok(Frame::Forbidden);
    };
    if !judge::postgres_compartments::holds(&standing, Over::Delete) {
        return Ok(Frame::Forbidden);
    }
    // Rule 2.
    let Some(entry) = find(&mut tx, &frame.compartment, true).await? else {
        return Ok(Frame::NotFound);
    };
    // Rule 3.
    let deletable = deletable(&mut tx, &entry).await?;
    if !judge::postgres_compartments::over(&standing, Over::Delete, &Facts { entry: &entry, deletable }) {
        return Ok(Frame::Forbidden);
    }
    // Rule 4.
    if !deletable {
        return Ok(Frame::NotDeletable);
    }
    // Rule 5.
    if daemon.live.connections_through(&entry.compartment).await != 0 {
        return Ok(Frame::InUse);
    }
    let tools = match (&entry.compartment, entry.kind) {
        (Compartment::Template { template, .. }, Of::DependencyTool) => dependency_tools::of_template(&mut tx, template, true).await?,
        _ => Vec::new(),
    };
    for tool in &tools {
        if load(&mut tx, daemon, tool).await?.is_some_and(|loaded| loaded.run.is_some()) {
            return Ok(Frame::InUse);
        }
    }
    // Rule 6.
    let mut removed = Vec::new();
    if let (Compartment::Template { template, .. }, Of::DependencyTool) = (&entry.compartment, entry.kind) {
        for tool in &tools {
            let owner = match tool.owner_agent {
                Some(id) => agents::by_id(&mut tx, id, false).await?,
                None => None,
            };
            if let Some(instance) = database::instance_of_dependency_tool(tool, owner.as_ref()) {
                database::remove(daemon, &mut tx, &instance).await?;
            }
        }
        removed = dependency_tools::delete_of_template(&mut tx, template).await?;
        dependencies::delete(&mut tx, template).await?;
    }
    database::remove(daemon, &mut tx, &entry.compartment).await?;
    tx.commit().await?;
    let gone: Vec<ToolKey> = removed.iter().map(|tool| ToolKey::Dependency(tool.id)).collect();
    eager::template_deleted(daemon, removed).await;
    for key in gone {
        tool_log::forget(daemon, key).await;
    }
    daemon.live.changed(Kind::PostgresCompartments);
    daemon.live.changed(Kind::DependencyTools);
    daemon.live.changed(Kind::StandaloneTools);
    Ok(Frame::Deleted)
}
