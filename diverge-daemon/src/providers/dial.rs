//! A provider the daemon dials, dialled for the daemon's life.

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::Duration;

use diverge_sdk::wire::connection::Connection;
use diverge_sdk::wire::handshake::{self, Handshook};
use tokio::net::TcpStream;
use tokio::task::JoinSet;
use tokio::time::timeout;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use diverge_sdk::shared::Identity;

use super::{Source, attach, keepalive, url};
use crate::containers::lifecycle::End;
use crate::daemon::Daemon;
use crate::judge::{Expected, Hold};

/// How long after a connection ends, or a dial fails, the next dial
/// is made: long enough that a provider that is down is not
/// hammered, short enough that one that is back is held soon.
const PAUSE: Duration = Duration::from_secs(5);

/// How long a dial waits for the provider's address to answer: its
/// TCP connection and its WebSocket upgrade.
const CONNECT: Duration = Duration::from_secs(20);

/// How long a dial waits for the handshake, once connected.
const HANDSHAKE: Duration = Duration::from_secs(30);

/// How long a dial waits for the provider to answer its info, once
/// the handshake is made.
const ATTACH: Duration = Duration::from_secs(30);

/// Dial the provider `identity` for as long as dial `number` is its
/// current one and its record enabled and outgoing. The rules:
///
/// 1. **Round after round.** Each [`round`] reads the record's
///    address, connects, makes the handshake, attaches, and serves the
///    connection to its end. Then the pause, and the next round.
/// 2. **Nothing ends the dial but its record.** A round that finds the
///    dial no longer current, or the record no longer enabled and
///    outgoing, ends the task, which forgets the dial when it is still
///    the current one; the daemon's stop ends it too, by the abort
///    handle [`Live`](crate::daemon::Live) keeps. A round that fails,
///    a round that panics, and a connection that ends are each only a
///    round over: the next is made.
/// 3. **Nothing waits for ever before a connection is held.** The
///    connect, the handshake and the attach each have a time, and one
///    that runs past it is a round that failed. A connection held is
///    never timed: it lasts as long as the provider is there, which
///    transport [`keepalive`] is what tells.
/// 4. **Why is kept.** What ended a round is kept as the provider's
///    status's `reason` until a round connects.
pub async fn dial(daemon: Arc<Daemon>, identity: Identity, number: u64) {
    rounds(&daemon, &identity, number).await;
    daemon.live.dial_ended(&identity, number).await;
}

/// The rounds of [`dial`], until one finds it should end.
async fn rounds(daemon: &Arc<Daemon>, identity: &Identity, number: u64) {
    loop {
        if !daemon.live.current_dial(identity, number).await {
            return;
        }
        let Some(address) = daemon.providers.dials(identity) else {
            return;
        };
        // Rule 2: a round on a task of its own, in a set that ends it
        // when this dial is ended, so that a panic in one is a round
        // that failed and not the dial gone.
        let mut one = JoinSet::new();
        one.spawn(round(Arc::clone(daemon), identity.clone(), address));
        let reason = match one.join_next().await {
            Some(Ok(reason)) => reason,
            Some(Err(error)) => format!("the dial failed unexpectedly: {error}"),
            None => "the dial ended with nothing said".to_string(),
        };
        // Rule 4.
        daemon.live.dial_failed(identity, reason).await;
        tokio::time::sleep(PAUSE).await;
    }
}

/// One round of [`dial`]: the provider at `address` connected to,
/// its handshake made as the connector — the daemon's own token
/// presented, the provider's verified against its own issuer, and
/// required to be exactly `identity`, which no other connection holds
/// and which it then holds — attached, and served until the
/// connection ends. What ended the round, in the daemon's words.
async fn round(daemon: Arc<Daemon>, identity: Identity, address: String) -> String {
    let socket = match timeout(CONNECT, keepalive::connect(&url(&address))).await {
        Ok(Ok(socket)) => socket,
        Ok(Err(error)) => return format!("{address} could not be reached: {error}"),
        Err(_) => return format!("{address} did not answer within {} seconds", CONNECT.as_secs()),
    };
    let ip = peer_ip(&socket);
    let expecting = daemon.authenticator.dialling(&identity);
    let (connection, peer) = match timeout(HANDSHAKE, handshake::connect(Connection::Outgoing(socket), &expecting, ip)).await {
        Ok(Ok(Handshook { connection, peer, .. })) => (connection, peer),
        Ok(Err(error)) => return format!("the handshake with {address} failed: {error}"),
        Err(_) => return format!("the handshake with {address} did not finish within {} seconds", HANDSHAKE.as_secs()),
    };
    // A dial that answers holds the identity: the hold goes with the
    // connection, however it ends.
    let hold = Hold::new(&daemon.authenticator, Expected::Provider, peer.clone());
    let attached = match timeout(ATTACH, attach(connection, peer, &daemon, hold, Source::Dialled(ip))).await {
        Ok(Ok(attached)) => attached,
        Ok(Err(error)) => return format!("{address} was connected to and not attached: {error}"),
        Err(_) => {
            // The attach was let go part way: what it reserved is
            // given back here, as it gives it back itself on every
            // path it ends by. Its hold went with it.
            daemon.live.forget_provider(&identity).await;
            daemon.live.disconnect_provider(&identity).await;
            return format!("{address} did not answer its info within {} seconds", ATTACH.as_secs());
        }
    };
    daemon.live.dial_connected(&identity).await;
    attached.serve().await;
    "the connection ended".to_string()
}

/// Start a dial of `identity`, its current one from now: an earlier
/// task for it ends at its next turn.
pub async fn start_dial(daemon: &Arc<Daemon>, identity: Identity) {
    let number = daemon.live.begin_dial(identity.clone()).await;
    let task = tokio::spawn(dial(Arc::clone(daemon), identity.clone(), number));
    daemon.live.dial_task(identity, number, task.abort_handle()).await;
}

/// Start a dial for every enabled, outgoing record: what the daemon
/// does once at start.
pub async fn dial_all(daemon: &Arc<Daemon>) {
    for record in daemon.providers.all().iter() {
        if record.enabled && record.direction.dialled().is_some() {
            start_dial(daemon, record.identity.clone()).await;
        }
    }
}

/// A change to `identity`'s record, committed and read into the
/// [`ProviderBook`](super::ProviderBook), taking effect on its
/// connection and its dial: the one place a connection is closed for
/// its record. What is held open on a connection that stays is
/// [`sync`](super::sync)'s. The rules:
///
/// 1. **A close.** When `end` is given, every run on the provider is
///    told it is ending so — its log's `undeployed` carries the reason —
///    and the connection to it, whichever end dialled, is closed: its
///    runs go down as on the socket ending.
/// 2. **The dial.** When the record is enabled and outgoing, it is
///    dialled: afresh after a close, or when no dial is current. When it
///    is not, its dial ends.
pub async fn reconcile(daemon: &Arc<Daemon>, identity: &Identity, end: Option<End>) {
    if let Some(end) = end {
        for run in daemon.live.agent_runs().await {
            if &run.provider == identity {
                run.end_as(end).await;
            }
        }
        for run in daemon.live.tool_runs().await {
            if &run.provider == identity {
                run.end_as(end).await;
            }
        }
        daemon.live.close_provider(identity).await;
    }
    if daemon.providers.dials(identity).is_some() {
        if end.is_some() || !daemon.live.dialling(identity).await {
            start_dial(daemon, identity.clone()).await;
        }
    } else {
        daemon.live.end_dial(identity).await;
    }
}

/// The peer's address, off the TCP stream under the WebSocket. With
/// no TLS in this crate the stream is always the plain one; anything
/// else, and a stream that will not say, is the unspecified address.
fn peer_ip(socket: &WebSocketStream<MaybeTlsStream<TcpStream>>) -> IpAddr {
    match socket.get_ref() {
        MaybeTlsStream::Plain(stream) => stream
            .peer_addr()
            .map(|address| address.ip())
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
        _ => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
    }
}
