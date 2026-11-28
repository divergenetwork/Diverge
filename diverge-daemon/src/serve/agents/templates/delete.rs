//! Deleting a agent template.

use std::sync::Arc;

use diverge_sdk::daemon::grant::agents_templates::Over;
use diverge_sdk::daemon::endpoints::agents::templates::delete::client::request;
use diverge_sdk::daemon::endpoints::agents::templates::delete::server::response::Frame;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;

use crate::daemon::{Daemon, Kind};
use crate::judge::{self, Standing, Who};
use crate::serve::reply;
use crate::database;
use crate::containers::{ToolKey, eager};
use crate::store::tools::template_attachments;
use crate::store::{self, agents_templates, connected_standalone_tools};
use diverge_sdk::daemon::endpoints::postgres::compartments::Kind as Of;
use super::in_use;

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
/// for a template the grants do not reach; `InUse` while some agent
/// was made from it; else the template deleted — its compartment of
/// the database dropped with its every table, every tool attached to
/// it detached and told its hold changed, its row kept, so one made
/// anew is this one again, with a fresh compartment and no
/// attachments. Its image's compartment stays: that goes by a delete
/// of its own.
async fn serve(frame: request::Frame, who: Who, daemon: &Arc<Daemon>) -> Result<Frame, store::Error> {
    let mut tx = daemon.store.begin().await?;
    let Some(standing) = Standing::of(&mut tx, who).await? else {
        return Ok(Frame::Forbidden);
    };
    if !judge::agents_templates::holds(&standing, Over::Delete) {
        return Ok(Frame::Forbidden);
    }
    let Some(record) = agents_templates::by_id(&mut tx, &frame.id, true).await? else {
        return Ok(Frame::NotFound);
    };
    let held = store::in_use::agents_templates(&mut tx).await?;
    let used = in_use(&held, &record);
    if !judge::agents_templates::over(&standing, Over::Delete, &record, used) {
        return Ok(Frame::Forbidden);
    }
    if used {
        return Ok(Frame::InUse);
    }
    database::remove(daemon, &mut tx, &database::template_of(Of::Agent, &record.id)).await?;
    let detached = template_attachments::clear_template(&mut tx, &record.id).await?;
    connected_standalone_tools::template_attachments::clear_template(&mut tx, &record.id).await?;
    agents_templates::delete(&mut tx, &record.id).await?;
    tx.commit().await?;
    for tool in detached {
        eager::detached(daemon, ToolKey::Record(tool)).await;
    }
    daemon.live.changed(Kind::AgentsTemplates);
    daemon.live.changed(Kind::PostgresCompartments);
    daemon.live.changed(Kind::StandaloneTools);
    daemon.live.changed(Kind::ConnectedStandaloneTools);
    Ok(Frame::Deleted)
}
