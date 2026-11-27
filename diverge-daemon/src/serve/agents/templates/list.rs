//! Listing agent templates, and keeping the list.

use std::collections::HashMap;
use std::future::Future;

use diverge_sdk::daemon::grant::agents_templates::Over;
use diverge_sdk::daemon::endpoints::agents::templates::list::client::request::{self, Filter};
use diverge_sdk::daemon::endpoints::agents::templates::list::server::response::{Frame, Listed};
use diverge_sdk::wire::server::scope_handle::ScopeHandle;

use crate::daemon::{Daemon, Kind};
use crate::judge::{self, Standing, Who, filter};
use crate::serve::stream::{self, Source};
use crate::serve::reply;
use crate::store::{self, agents_templates, connected_standalone_tools, tools};
use diverge_sdk::daemon::key;
use super::in_use;

/// Send the list and its changes, then finish the scope.
pub async fn handle(scope: ScopeHandle, frame: request::Frame, who: Who, daemon: &Daemon) {
    if let Err(error) = serve(&scope, frame, who, daemon).await {
        reply::reply(&scope, &Frame::Error(reply::failure(&error))).await;
    }
    scope.send_response_finish().await;
}

/// One `Forbidden` with no `list` grant at all; else every template
/// any `list` grant reaches that the request's filter lets through,
/// oldest first, the first `count` of them, one frame each, then the
/// word that the list is whole — and from then on each template added,
/// changed or removed as templates are made, tagged and deleted and as
/// the agents made from them come and go, until the client cancels.
///
/// The rules of the request's `arguments`:
///
/// 1. `false`: nothing more is asked of the caller, and no template
///    listed carries its template's arguments.
/// 2. `true`, at the open: every template the list shows now — reached
///    by a `list` grant and let through by the filter, whatever
///    `count` then keeps of them — must be reached by an `arguments`
///    grant as well. One that is not is the list's `Forbidden`, and
///    nothing is listed.
/// 3. `true`, kept open: a template that comes to be shown later and is
///    reached by no `arguments` grant is not shown on this list at
///    all, since it is outside what this list may show; one already
///    shown that leaves that reach is removed from it.
async fn serve(scope: &ScopeHandle, frame: request::Frame, who: Who, daemon: &Daemon) -> Result<(), store::Error> {
    let standing = {
        let mut conn = daemon.store.acquire().await?;
        Standing::of(&mut conn, who).await?
    };
    let Some(standing) = standing else {
        reply::reply(scope, &Frame::Forbidden).await;
        return Ok(());
    };
    if !judge::agents_templates::holds(&standing, Over::List) {
        reply::reply(scope, &Frame::Forbidden).await;
        return Ok(());
    }
    let source = Templates {
        daemon,
        standing: &standing,
        filter: &frame.filter,
        arguments: frame.arguments,
    };
    // Rule 2 of `arguments`.
    if frame.arguments && source.rows(true).await?.is_none() {
        reply::reply(scope, &Frame::Forbidden).await;
        return Ok(());
    }
    stream::listing(scope, daemon, &[Kind::AgentsTemplates, Kind::Agents, Kind::StandaloneTools, Kind::ConnectedStandaloneTools], frame.count, &source, Frame::from)
    .await
}

/// The templates as the caller may list them now.
struct Templates<'a> {
    daemon: &'a Daemon,
    standing: &'a Standing,
    filter: &'a Filter,
    /// Whether the request asked for the templates' arguments.
    arguments: bool,
}

impl Templates<'_> {
    /// The templates the list shows now, each as it reports it. With
    /// `strict`, `None` where the request's `arguments` is `true` and
    /// one of them is reached by no `arguments` grant: rule 2. Without
    /// it, such a one is left out: rule 3.
    async fn rows(&self, strict: bool) -> Result<Option<Vec<(String, Listed)>>, store::Error> {
        let (all, held, attached) = {
            let mut conn = self.daemon.store.acquire().await?;
            let all = agents_templates::all(&mut conn).await?;
            let held = store::in_use::agents_templates(&mut conn).await?;
            let keys: HashMap<_, _> = tools::all(&mut conn).await?.into_iter().map(|tool| (tool.id, tool.key())).collect();
            let mut attached: HashMap<String, Vec<key::Tool>> = HashMap::new();
            for attachment in tools::template_attachments::all(&mut conn).await? {
                if let Some(key) = keys.get(&attachment.tool) {
                    attached.entry(attachment.template).or_default().push(key.clone());
                }
            }
            // The connected tools after the standalone ones.
            let connected: HashMap<_, _> = connected_standalone_tools::all(&mut conn).await?.into_iter().map(|tool| (tool.id, tool.key())).collect();
            for attachment in connected_standalone_tools::template_attachments::all(&mut conn).await? {
                if let Some(key) = connected.get(&attachment.tool) {
                    attached.entry(attachment.template).or_default().push(key.clone());
                }
            }
            (all, held, attached)
        };
        let mut listed = Vec::new();
        for record in &all {
            let in_use = in_use(&held, record);
            if !judge::agents_templates::over(self.standing, Over::List, record, in_use) || !filter::agents_templates::test(self.filter, record, in_use) {
                continue;
            }
            if self.arguments && !judge::agents_templates::over(self.standing, Over::Arguments, record, in_use) {
                if strict {
                    return Ok(None);
                }
                continue;
            }
            listed.push((record.id.clone(), record.report(attached.get(&record.id).cloned().unwrap_or_default(), self.arguments)));
        }
        Ok(Some(listed))
    }
}

impl Source for Templates<'_> {
    type Key = String;
    type Item = Listed;
    type Error = store::Error;

    fn read(&self) -> impl Future<Output = Result<Vec<(String, Listed)>, store::Error>> + Send {
        async move { Ok(self.rows(false).await?.unwrap_or_default()) }
    }
}
