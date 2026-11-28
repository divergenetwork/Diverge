//! Deleting an agent.

use std::sync::Arc;

use diverge_sdk::daemon::endpoints::agents::delete::client::request;
use diverge_sdk::daemon::endpoints::agents::delete::server::response::Frame;
use diverge_sdk::daemon::grant::agents::Over;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;

use super::{Failure, state};
use crate::containers;
use crate::daemon::{Daemon, Kind};
use crate::judge::{self, Standing, Who};
use crate::logs;
use crate::serve::reply;
use crate::database;
use crate::store::{agents, dependency_tools, tools};

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
/// for an agent the grants do not reach; `Active` while it is active,
/// or while any dependency tool it owns is — a call or a read that
/// tool was asked still unanswered; else the agent gone — its container stopped if it was up,
/// its attachments with it, its own compartment of the database
/// dropped with its every table, the dependency tools it owns gone
/// with theirs, each one's run stopped, every other dependency tool and every tool on record
/// it was attached to told,
/// its name free, its MCP session in every tool running forgotten,
/// every watch on its log ended and the log removed.
/// Its template's compartment and its image's stay: they are every
/// agent's of the template, and of the image.
async fn serve(frame: request::Frame, who: Who, daemon: &Arc<Daemon>) -> Result<Frame, Failure> {
    let mut tx = daemon.store.begin().await?;
    let Some(standing) = Standing::of(&mut tx, who).await? else {
        return Ok(Frame::Forbidden);
    };
    if !judge::agents::holds(&standing, Over::Delete) {
        return Ok(Frame::Forbidden);
    }
    let Some(agent) = agents::by_reference(&mut tx, &frame.agent, true).await? else {
        return Ok(Frame::NotFound);
    };
    let state = state(daemon, agent.id).await;
    if !judge::agents::over(&standing, Over::Delete, &agent, state) {
        return Ok(Frame::Forbidden);
    }
    if state.active {
        return Ok(Frame::Active);
    }
    // The tools it owns go with it, and one that is working is not
    // taken out from under its ask.
    for tool in dependency_tools::of_owner_agent(&mut tx, agent.id).await? {
        if daemon.live.is_busy(containers::ToolKey::Dependency(tool.id)).await {
            return Ok(Frame::Active);
        }
    }
    let attached = dependency_tools::of_agent(&mut tx, agent.id).await?;
    let detached = tools::attachments::of_agent(&mut tx, agent.id).await?;
    for tool in dependency_tools::of_owner_agent(&mut tx, agent.id).await? {
        if let Some(instance) = database::instance_of_dependency_tool(&tool, Some(&agent)) {
            database::remove(daemon, &mut tx, &instance).await?;
        }
    }
    database::remove(daemon, &mut tx, &database::instance_of_agent(&agent)).await?;
    let removed = dependency_tools::delete_of_owner_agent(&mut tx, agent.id).await?;
    agents::delete(&mut tx, agent.id).await?;
    tx.commit().await?;
    containers::stop_agent(daemon, agent.id).await;
    let left = attached
        .iter()
        .filter(|tool| !removed.iter().any(|gone| gone.id == tool.id))
        .map(|tool| tool.id)
        .collect();
    let gone: Vec<containers::ToolKey> = removed.iter().map(|tool| containers::ToolKey::Dependency(tool.id)).collect();
    containers::eager::agent_deleted(daemon, removed, left, detached).await;
    for key in gone {
        containers::tool_log::forget(daemon, key).await;
    }
    containers::tool_log::forget_agent(daemon, agent.key()).await;
    daemon.live.end_log(agent.id).await;
    logs::remove(&daemon.logs, agent.id).await?;
    daemon.live.changed(Kind::Agents);
    daemon.live.changed(Kind::StandaloneTools);
    daemon.live.changed(Kind::ConnectedStandaloneTools);
    daemon.live.changed(Kind::DependencyTools);
    daemon.live.changed(Kind::Volumes);
    daemon.live.changed(Kind::PostgresCompartments);
    Ok(Frame::Deleted)
}
