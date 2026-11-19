//! Making a request of others, and waiting for its answer.

use std::sync::Arc;

use diverge_sdk::daemon::endpoints::requests::RequestId;
use diverge_sdk::daemon::endpoints::requests::create::client::request::{self, Only, To};
use diverge_sdk::daemon::endpoints::requests::create::client::channel_request;
use diverge_sdk::daemon::endpoints::requests::create::server::response::{Created, Delivered, Frame, Responded};
use diverge_sdk::daemon::grant;
use diverge_sdk::wire::decode::Decode as _;
use diverge_sdk::wire::frame::client::ClientFrame;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;
use sqlx::PgConnection;

use super::eligible;
use crate::daemon::Daemon;
use crate::judge::{self, Standing, Who};
use crate::requests::{Became, Party};
use crate::serve::{dependency_tools, reply, standalone_tools};
use crate::store::{self, accounts, dependency_tools as dependencies, tools};

/// Serve the create to its end, and finish the scope.
pub async fn handle(scope: ScopeHandle, frame: request::Frame, who: Who, daemon: &Arc<Daemon>) {
    if let Err(error) = serve(&scope, frame, who, daemon).await {
        reply::reply(&scope, &Frame::Error(reply::failure(&error))).await;
    }
    scope.send_response_finish().await;
}

/// The rules:
///
/// 1. `Forbidden` with no `request` grant at all: over accounts, over
///    standalone tools, or over dependency tools.
/// 2. **Whom it names**, judged by [`judged`] before the request
///    exists: `NotFound`, `Forbidden` or `Error`, each alone.
/// 3. **Created.** An id is drawn at random, the request is put in the
///    hub, and `Created` is sent.
/// 4. **Sent to who is listening now**, on a task of its own, so that
///    the scope hears its cancel while the listeners are judged: each
///    party listening at the create is judged by [`eligible`] and
///    delivered. Every listener that opens after finds the request by
///    its own look. Either way a party is sent it once, and each
///    delivery is one `Delivered`.
/// 5. **Answered.** `Responded`, as the hub says it: the request is
///    over, and out of the hub already.
/// 6. **Taken back.** The caller's `Cancel`, the caller gone, or the
///    caller reading no more withdraws the request from everyone it
///    was sent to. The scope finishes with no `Responded`.
async fn serve(scope: &ScopeHandle, frame: request::Frame, who: Who, daemon: &Arc<Daemon>) -> Result<(), store::Error> {
    let mut conn = daemon.store.acquire().await?;
    let Some(standing) = Standing::of(&mut conn, who).await? else {
        reply::reply(scope, &Frame::Forbidden).await;
        return Ok(());
    };
    // Rule 1.
    let holds = judge::accounts::holds(&standing, grant::accounts::Over::Request)
        || judge::standalone_tools::holds(&standing, grant::standalone_tools::Over::Request)
        || judge::dependency_tools::holds(&standing, grant::dependency_tools::Over::Request);
    if !holds {
        reply::reply(scope, &Frame::Forbidden).await;
        return Ok(());
    }
    // Rule 2.
    if let To::Only(only) = &frame.to {
        if let Some(refusal) = judged(&mut conn, daemon, &standing, only).await? {
            reply::reply(scope, &refusal).await;
            return Ok(());
        }
    }
    drop(conn);
    scope.allowed().await;
    // Rule 3.
    let id = RequestId(uuid::Uuid::new_v4().to_string());
    let (mut became, listening) = daemon.live.requests.create(id.clone(), frame.request, frame.to.clone(), standing.clone()).await;
    reply::reply(scope, &Frame::Created(Created { id: id.clone() })).await;
    // Rule 4.
    let sending = tokio::spawn(send(Arc::clone(daemon), id.clone(), frame.to, standing, listening));
    let answered = loop {
        tokio::select! {
            word = became.recv() => match word {
                Some(Became::Delivered(to)) => reply::reply(scope, &Frame::Delivered(Delivered { to })).await,
                // Rule 5.
                Some(Became::Responded { response, from }) => {
                    reply::reply(scope, &Frame::Responded(Responded { response, from })).await;
                    break true;
                }
                None => break true,
            },
            // Rule 6.
            request = scope.recv_channel_request() => {
                let Some(bytes) = request else {
                    break false;
                };
                let Ok(ClientFrame::ChannelRequest { payload, .. }) = ClientFrame::decode(&bytes) else {
                    continue;
                };
                if let Ok(channel_request::Frame::Cancel) = channel_request::Frame::decode(payload) {
                    break false;
                }
            }
            () = scope.responses_gone() => break false,
        }
    };
    sending.abort();
    if !answered {
        daemon.live.requests.withdraw(&id).await;
    }
    Ok(())
}

/// Rule 4 of [`serve`]: every party of `listening` judged and, where
/// the request is made of it, delivered. A store that fails ends the
/// sending; a party not reached now is reached by its own next look.
async fn send(daemon: Arc<Daemon>, id: RequestId, to: To, standing: Standing, listening: Vec<Party>) {
    let Ok(mut conn) = daemon.store.acquire().await else {
        return;
    };
    for party in &listening {
        match eligible(&mut conn, &daemon, party, &to, &standing).await {
            Ok(true) => daemon.live.requests.deliver(&id, party).await,
            Ok(false) => {}
            Err(_) => return,
        }
    }
}

/// Rule 2 of [`serve`]: the answer that refuses a request naming
/// `only`, or `None` where everything it names may be asked. The
/// rules:
///
/// 1. **It names somebody.** Every member empty is `Error`.
/// 2. **An account named** is one there is, or an administrator the
///    configuration names, else `NotFound`; and one the `request`
///    grant over accounts reaches, else `Forbidden`.
/// 3. **A standalone tool, and a dependency tool, named** is one there
///    is, else `NotFound`; and one the `request` grant for its kind
///    reaches, else `Forbidden`.
/// 4. **A tag named** is one whose every record, as of now, the
///    `request` grant for its kind reaches, else `Forbidden`: a tag is
///    named whole, and a caller that may ask some who carry it and not
///    others may not name it. A tag nobody carries names nobody, and
///    is allowed.
/// 5. Connection is not asked: whether an account is connected is
///    read as it is, and a tool has none.
async fn judged(conn: &mut PgConnection, daemon: &Arc<Daemon>, standing: &Standing, only: &Only) -> Result<Option<Frame>, store::Error> {
    // Rule 1.
    if only.is_empty() {
        return Ok(Some(Frame::Error(reply::failure(&"the request names nobody it is made of"))));
    }
    let action = grant::accounts::Over::Request;
    // Rule 2.
    for identity in &only.accounts {
        match accounts::by_identity(conn, identity, false).await? {
            Some(account) => {
                if !judge::accounts::over(standing, action, &account, daemon.live.is_connected(account.id).await) {
                    return Ok(Some(Frame::Forbidden));
                }
            }
            None if daemon.authenticator.is_admin(identity) => {
                let connected = daemon.live.admins_connected().await.contains(identity);
                if !judge::accounts::over_admin(standing, action, identity, connected) {
                    return Ok(Some(Frame::Forbidden));
                }
            }
            None => return Ok(Some(Frame::NotFound)),
        }
    }
    // Rule 4, accounts.
    if !only.account_tags.is_empty() {
        for account in accounts::all(conn).await? {
            if only.account_tags.iter().any(|tag| account.tags.contains(tag)) && !judge::accounts::over(standing, action, &account, daemon.live.is_connected(account.id).await) {
                return Ok(Some(Frame::Forbidden));
            }
        }
    }
    // Rules 3 and 4, standalone tools.
    let action = grant::standalone_tools::Over::Request;
    for reference in &only.standalone_tools {
        let Some(tool) = tools::by_reference(conn, reference, false).await? else {
            return Ok(Some(Frame::NotFound));
        };
        if !standalone_tools::reaches(conn, daemon, standing, action, &tool).await? {
            return Ok(Some(Frame::Forbidden));
        }
    }
    if !only.standalone_tool_tags.is_empty() {
        for tool in tools::all(conn).await? {
            if only.standalone_tool_tags.iter().any(|tag| tool.tags.contains(tag)) && !standalone_tools::reaches(conn, daemon, standing, action, &tool).await? {
                return Ok(Some(Frame::Forbidden));
            }
        }
    }
    // Rules 3 and 4, dependency tools.
    let action = grant::dependency_tools::Over::Request;
    for reference in &only.dependency_tools {
        let Some(tool) = dependencies::by_reference(conn, reference, false).await? else {
            return Ok(Some(Frame::NotFound));
        };
        if !dependency_tools::reaches(conn, daemon, standing, action, &tool).await? {
            return Ok(Some(Frame::Forbidden));
        }
    }
    if !only.dependency_tool_tags.is_empty() {
        for tool in dependencies::all(conn).await? {
            if only.dependency_tool_tags.iter().any(|tag| tool.tags.contains(tag)) && !dependency_tools::reaches(conn, daemon, standing, action, &tool).await? {
                return Ok(Some(Frame::Forbidden));
            }
        }
    }
    Ok(None)
}
