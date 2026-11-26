//! Tagging a compartment.

use diverge_sdk::daemon::endpoints::postgres::compartments::tag::client::request;
use diverge_sdk::daemon::endpoints::postgres::compartments::tag::server::response::Frame;
use diverge_sdk::daemon::grant::Tagging;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;

use super::{deletable, find};
use crate::daemon::{Daemon, Kind};
use crate::judge::filter::postgres_compartments::Facts;
use crate::judge::{self, Standing, Who};
use crate::serve::reply;
use crate::store::{self, postgres_compartments, tags};

/// Answer the tag and finish the scope.
pub async fn handle(scope: ScopeHandle, frame: request::Frame, who: Who, daemon: &Daemon) {
    let answer = match serve(frame, who, daemon).await {
        Ok(answer) => answer,
        Err(error) => Frame::Error(reply::failure(&error)),
    };
    reply::reply(&scope, &answer).await;
    scope.send_response_finish().await;
}

/// `Forbidden` with no `tag` grant at all; `NotFound`; `Forbidden`
/// for a compartment the grants do not reach, or a tag the grant does
/// not cover; else the tags on the compartment.
async fn serve(frame: request::Frame, who: Who, daemon: &Daemon) -> Result<Frame, store::Error> {
    let mut tx = daemon.store.begin().await?;
    let Some(standing) = Standing::of(&mut tx, who).await? else {
        return Ok(Frame::Forbidden);
    };
    if !judge::postgres_compartments::holds_tagging(&standing, Tagging::Tag) {
        return Ok(Frame::Forbidden);
    }
    let Some(entry) = find(&mut tx, &frame.compartment, true).await? else {
        return Ok(Frame::NotFound);
    };
    let deletable = deletable(&mut tx, &entry).await?;
    if !judge::postgres_compartments::tagging(&standing, Tagging::Tag, &Facts { entry: &entry, deletable }, &frame.tags) {
        return Ok(Frame::Forbidden);
    }
    postgres_compartments::set_tags(&mut tx, &entry.role, &tags::with(&entry.tags, &frame.tags)).await?;
    tx.commit().await?;
    daemon.live.changed(Kind::PostgresCompartments);
    Ok(Frame::Tagged)
}
