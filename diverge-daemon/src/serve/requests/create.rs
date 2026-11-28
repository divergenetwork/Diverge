//! Making a request of others, and waiting for its answer.

use std::sync::Arc;

use diverge_sdk::daemon::endpoints::requests::RequestId;
use diverge_sdk::daemon::endpoints::requests::create::client::request::{self, To};
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
use crate::serve::reply;
use crate::store::{self, accounts, agents, dependency_tools as dependencies, tools};

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
///    agents, over standalone tools, or over dependency tools.
/// 2. **Made of somebody**, by [`sendable`], before the request
///    exists: `NoRecipient` where no account, no agent and no tool there is is
///    both addressed and reached. Nothing else about whom it names
///    refuses it.
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
        || judge::agents::holds(&standing, grant::agents::Over::Request)
        || judge::standalone_tools::holds(&standing, grant::standalone_tools::Over::Request)
        || judge::dependency_tools::holds(&standing, grant::dependency_tools::Over::Request);
    if !holds {
        reply::reply(scope, &Frame::Forbidden).await;
        return Ok(());
    }
    // Rule 2.
    if !sendable(&mut conn, daemon, &standing, &frame.to).await? {
        reply::reply(scope, &Frame::NoRecipient).await;
        return Ok(());
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

/// Rule 2 of [`serve`]: whether the request made `to` those is made
/// of anybody. The rules:
///
/// 1. **Anybody there is.** Every account, every administrator with
///    no account, every agent, every standalone tool and every
///    dependency tool is
///    asked of [`eligible`], which is the judgement a delivery is
///    made by: addressed by `to`, and reached by the `request` grant
///    for its kind. So a request passes here exactly where some party
///    would be sent it were it listening.
/// 2. **One is enough.** The first that passes answers, and the rest
///    are not read.
/// 3. **Listening is not asked.** Whether a party is connected, or has
///    a listen open, changes nothing here.
/// 4. **Nothing is told of what did not pass.** A name that names
///    nothing, a party addressed and not reached, and a tag nobody
///    reached carries are one answer, and the caller is not told which.
async fn sendable(conn: &mut PgConnection, daemon: &Arc<Daemon>, standing: &Standing, to: &To) -> Result<bool, store::Error> {
    let accounts = accounts::all(conn).await?;
    for account in &accounts {
        if eligible(conn, daemon, &Party::Account(account.id), to, standing).await? {
            return Ok(true);
        }
    }
    let admins: Vec<_> = daemon.authenticator.admins().filter(|admin| !accounts.iter().any(|account| account.identity == **admin)).cloned().collect();
    for admin in admins {
        if eligible(conn, daemon, &Party::Admin(admin), to, standing).await? {
            return Ok(true);
        }
    }
    for agent in agents::all(conn).await? {
        if eligible(conn, daemon, &Party::Agent(agent.id), to, standing).await? {
            return Ok(true);
        }
    }
    for tool in tools::all(conn).await? {
        if eligible(conn, daemon, &Party::Standalone(tool.id), to, standing).await? {
            return Ok(true);
        }
    }
    for tool in dependencies::all(conn).await? {
        if eligible(conn, daemon, &Party::Dependency(tool.id), to, standing).await? {
            return Ok(true);
        }
    }
    Ok(false)
}
