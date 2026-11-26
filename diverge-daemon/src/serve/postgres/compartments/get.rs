//! Getting one compartment.

use diverge_sdk::daemon::endpoints::postgres::compartments::get::client::request;
use diverge_sdk::daemon::endpoints::postgres::compartments::get::server::response::Frame;
use diverge_sdk::daemon::grant::postgres_compartments::Over;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;

use super::{deletable, find};
use crate::daemon::Daemon;
use crate::judge::filter::postgres_compartments::Facts;
use crate::judge::{self, Standing, Who};
use crate::serve::reply;
use crate::store;

/// Answer the get and finish the scope.
pub async fn handle(scope: ScopeHandle, frame: request::Frame, who: Who, daemon: &Daemon) {
    let answer = match serve(frame, who, daemon).await {
        Ok(answer) => answer,
        Err(error) => Frame::Error(reply::failure(&error)),
    };
    reply::reply(&scope, &answer).await;
    scope.send_response_finish().await;
}

/// `Forbidden` with no `get` grant at all; `NotFound`; `Forbidden` for
/// a compartment the grants do not reach; else the compartment, as a
/// list reports it.
async fn serve(frame: request::Frame, who: Who, daemon: &Daemon) -> Result<Frame, store::Error> {
    let mut conn = daemon.store.acquire().await?;
    let Some(standing) = Standing::of(&mut conn, who).await? else {
        return Ok(Frame::Forbidden);
    };
    if !judge::postgres_compartments::holds(&standing, Over::Get) {
        return Ok(Frame::Forbidden);
    }
    let Some(entry) = find(&mut conn, &frame.compartment, false).await? else {
        return Ok(Frame::NotFound);
    };
    let deletable = deletable(&mut conn, &entry).await?;
    if !judge::postgres_compartments::over(&standing, Over::Get, &Facts { entry: &entry, deletable }) {
        return Ok(Frame::Forbidden);
    }
    let connections = daemon.live.connections_through(&entry.compartment).await;
    Ok(Frame::Found(entry.report(deletable, connections)))
}
