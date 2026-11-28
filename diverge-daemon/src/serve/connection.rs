//! One accepted socket, for its life.

use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;

use diverge_sdk::wire::connection::Connection;
use diverge_sdk::wire::handshake::{self, Handshook};
use diverge_sdk::wire::server::received::Received;
use diverge_sdk::wire::server::session::Session;
use futures_util::StreamExt as _;
use tokio::sync::watch;
use tokio::task::JoinSet;

use super::{Turns, dispatch};
use crate::containers::Inflight;
use crate::daemon::{Connected, Daemon};
use crate::store::AccountId;
use crate::judge::{Admitted, Expected, Hold, Who};
use crate::providers;

/// Serve `connection` from `address` until it ends.
///
/// The first thing either end says is who it is: the
/// [`handshake`](diverge_sdk::wire::handshake), made here
/// as the connectee, since a client and a provider dial the one port
/// and open the one way, and which half of the wire the daemon holds
/// toward the socket is decided by what the verified identity is.
/// The [`DaemonAuthenticator`](crate::judge::DaemonAuthenticator)
/// verifies the connector's token and refuses an identity that is
/// neither an account nor an accepted provider, and a refusal — or a
/// store that could not be asked — is the connection closed without a
/// word. A provider admitted is [`attach`](providers::attach)ed, where
/// the daemon is the caller — and where a second connection under an
/// identity a connection holds is dropped the same way; a client
/// admitted is served here, as a client is.
///
/// The identity the handshake held as a provider's is the connection's
/// [`Hold`] from the handshake on, handed to the attach, which gives it
/// back at the connection's end; on every other path — a client after
/// all, nothing, an error, the task aborted — it is given back here. A
/// provider the handshake did not hold is not attached.
pub async fn connection(connection: Connection, address: IpAddr, daemon: Arc<Daemon>) {
    let took = tokio::sync::watch::channel(false).0;
    let Ok(Handshook { connection, peer, .. }) = handshake::accept(connection, &daemon.authenticator.admitting(&took), address).await else {
        return;
    };
    let hold = (*took.borrow()).then(|| Hold::new(&daemon.authenticator, Expected::Admitted, peer.clone()));
    match (daemon.authenticator.admit(&peer).await, hold) {
        (Ok(Some(Admitted::Provider(identity))), Some(hold)) => {
            if let Ok(attached) = providers::attach(connection, identity, &daemon, hold, providers::Source::Accepted(address)).await {
                attached.serve().await;
            }
        }
        (Ok(Some(Admitted::Client(who))), hold) => {
            if let Some(hold) = hold {
                hold.release().await;
            }
            client(connection, who, daemon, None).await;
        }
        (_, hold) => {
            if let Some(hold) = hold {
                hold.release().await;
            }
        }
    }
}

/// What a tool's daemon connection holds of the tool's run: given to
/// [`client`] for a connection a tool's container opened, and to
/// nothing else. The rules:
///
/// 1. **Every scope holds the run.** Each request the tool's program
///    puts to the daemon is counted in `scopes` from the moment it is
///    read until its task ends — answered, cancelled, or its
///    connection gone — whatever it asks, an endless stream included:
///    rule 7 of [`idle_tool`](crate::containers::idle_tool). A tool
///    that wants to go idle ends its scopes.
/// 2. **A scope's end is a use.** `touched` is noted as it is counted
///    out, so the run's idle clock counts from there.
/// 3. **The count is the run's.** `scopes` is the one the run's
///    answerer holds and its run reads, so a scope is never counted
///    on another run of the same tool.
#[derive(Debug, Clone)]
pub struct Held {
    /// The run's open daemon scopes.
    pub scopes: Arc<Inflight>,
    /// Where a use of the run's container is noted.
    pub touched: watch::Sender<Instant>,
}

/// Serve a client admitted as `who`, past its handshake, until the
/// socket ends.
///
/// The connection counts itself in as its account — a tool's, served
/// under the grants it declared, is no account's, and with `held`
/// each scope it opens holds the tool's container up until the scope
/// ends — and every
/// scope the client opens is read as a request and handed to
/// [`dispatch`] on a task of its own, so a slow answer never holds
/// the socket; a frame that is not a request ends the connection.
/// Every request is given its [`Turn`](super::Turn) here first, as it
/// is read and before its task is spawned — a link in the
/// connection's chain, in the order it was read — which is what a
/// message to an agent is admitted in, whatever order the tasks'
/// lookups finish in; every other request lets its turn go at once.
/// When the
/// socket is gone it is dropped first, which tells every scope still
/// being served that its feed has closed, and then every task still
/// answering is waited for, since an answer composed is an answer
/// sent, and the connection counts itself out.
pub async fn client(connection: Connection, who: Who, daemon: Arc<Daemon>, held: Option<Held>) {
    let mut session = Session::new(connection);
    match &who {
        Who::Account(account) => {
            if daemon.live.enter(*account).await {
                connected_at(&daemon, *account).await;
            }
        }
        Who::Admin(identity) => daemon.live.enter_admin(identity).await,
        Who::Tool(_) => {}
    }
    // The connection's closer: an account's and an administrator's may
    // be told to close — see `accounts disconnect` — and a tool's has
    // none.
    let connected = match &who {
        Who::Account(account) => Some(Connected::Account(*account)),
        Who::Admin(identity) => Some(Connected::Admin(identity.clone())),
        Who::Tool(_) => None,
    };
    let (key, mut closing) = match &connected {
        Some(connected) => {
            let (key, closing) = daemon.live.open(connected.clone()).await;
            (Some(key), closing)
        }
        None => (None, tokio::sync::watch::channel(false).1),
    };
    let mut scopes = JoinSet::new();
    let mut turns = Turns::new();
    loop {
        let received = tokio::select! {
            received = session.next() => received,
            // Told to close: the session ends as when the socket drops.
            _ = closing.wait_for(|closing| *closing), if key.is_some() => break,
        };
        let Some(received) = received else {
            break;
        };
        while scopes.try_join_next().is_some() {}
        let Received::Request(payload, scope) = received else {
            break;
        };
        let turn = turns.next();
        let daemon = Arc::clone(&daemon);
        let who = who.clone();
        // A tool's own scope holds the tool's run for the scope's
        // life, whatever it asks: counted in here, as the scope is
        // read, and out when its task ends, which is a use of the
        // container, so the idle clock counts from there.
        let using = held.as_ref().map(|held| held.scopes.enter_use(held.touched.clone()));
        scopes.spawn(async move {
            let _using = using;
            dispatch(scope, &payload, turn, who, &daemon).await;
        });
    }
    drop(session);
    while scopes.join_next().await.is_some() {}
    if let (Some(connected), Some(key)) = (&connected, key) {
        daemon.live.close(connected, key).await;
    }
    match &who {
        Who::Account(account) => {
            if daemon.live.leave(*account).await {
                connected_at(&daemon, *account).await;
            }
        }
        Who::Admin(identity) => daemon.live.leave_admin(identity).await,
        Who::Tool(_) => {}
    }
}

/// Keep now as the account's `last_connected`: its first connection
/// began, or its last ended. A time that cannot be kept fails nothing.
async fn connected_at(daemon: &Daemon, account: AccountId) {
    if let Ok(mut conn) = daemon.store.acquire().await {
        let _ = crate::store::accounts::set_connected(&mut conn, account, chrono::Utc::now()).await;
    }
}
