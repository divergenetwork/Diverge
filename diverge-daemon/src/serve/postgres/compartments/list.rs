//! Listing compartments, and keeping the list.

use std::future::Future;

use diverge_sdk::daemon::endpoints::postgres::compartments::Compartment;
use diverge_sdk::daemon::endpoints::postgres::compartments::list::client::request::{self, Filter};
use diverge_sdk::daemon::endpoints::postgres::compartments::list::server::response::{Frame, Listed as Item};
use diverge_sdk::daemon::grant::postgres_compartments::Over;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;

use crate::daemon::{Daemon, Kind};
use crate::judge::filter::postgres_compartments::Facts;
use crate::judge::{self, Standing, Who, filter};
use crate::serve::stream::{self, Source};
use crate::serve::reply;
use crate::store::postgres_compartments::{self, Holds};
use crate::store;

/// Send the list and its changes, then finish the scope.
pub async fn handle(scope: ScopeHandle, frame: request::Frame, who: Who, daemon: &Daemon) {
    if let Err(error) = serve(&scope, frame, who, daemon).await {
        reply::reply(&scope, &Frame::Error(reply::failure(&error))).await;
    }
    scope.send_response_finish().await;
}

/// One `Forbidden` with no `list` grant at all; else every compartment
/// any `list` grant reaches that the request's filter lets through,
/// oldest first, the first `count` of them, one frame each, then the
/// word that the list is whole; and from then on each compartment
/// added, changed or removed — as records are made and deleted, as
/// tags change, as what keeps one from being deleted comes and goes,
/// and as connections open and close — until the client cancels.
async fn serve(scope: &ScopeHandle, frame: request::Frame, who: Who, daemon: &Daemon) -> Result<(), store::Error> {
    let standing = {
        let mut conn = daemon.store.acquire().await?;
        Standing::of(&mut conn, who).await?
    };
    let Some(standing) = standing else {
        reply::reply(scope, &Frame::Forbidden).await;
        return Ok(());
    };
    if !judge::postgres_compartments::holds(&standing, Over::List) {
        reply::reply(scope, &Frame::Forbidden).await;
        return Ok(());
    }
    let source = Listed {
        daemon,
        standing: &standing,
        filter: &frame.filter,
    };
    let kinds = [
        Kind::PostgresCompartments,
        Kind::Postgres,
        Kind::Agents,
        Kind::AgentsTemplates,
        Kind::StandaloneTools,
        Kind::StandaloneToolsTemplates,
        Kind::DependencyTools,
    ];
    stream::listing(scope, daemon, &kinds, frame.count, &source, Frame::from).await
}

/// The compartments as the caller may list them now.
struct Listed<'a> {
    daemon: &'a Daemon,
    standing: &'a Standing,
    filter: &'a Filter,
}

impl Source for Listed<'_> {
    type Key = Compartment;
    type Item = Item;
    type Error = store::Error;

    fn read(&self) -> impl Future<Output = Result<Vec<(Compartment, Item)>, store::Error>> + Send {
        async move {
            let (all, holds) = {
                let mut conn = self.daemon.store.acquire().await?;
                (postgres_compartments::all(&mut conn).await?, Holds::load(&mut conn).await?)
            };
            let open = self.daemon.live.connections().await;
            Ok(all
                .iter()
                .map(|entry| Facts {
                    entry,
                    deletable: holds.deletable(entry),
                })
                .filter(|facts| judge::postgres_compartments::over(self.standing, Over::List, facts))
                .filter(|facts| filter::postgres_compartments::test(self.filter, facts))
                .map(|facts| {
                    let connections = open.iter().filter(|(_, _, compartment, _)| *compartment == facts.entry.compartment).count();
                    (
                        facts.entry.compartment.clone(),
                        facts.entry.report(facts.deletable, u64::try_from(connections).unwrap_or(u64::MAX)),
                    )
                })
                .collect())
        }
    }
}
