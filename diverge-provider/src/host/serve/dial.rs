//! One peer the provider dials, for the provider's life.

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use diverge_sdk::wire::connection::Connection;
use diverge_sdk::wire::handshake::{self, Handshook};
use tokio::net::TcpStream;
use tokio::time::{Duration, sleep, timeout};

/// How long a dial waits for the peer's address to answer.
const CONNECT: Duration = Duration::from_secs(20);

/// How long a handshake may take, dialled or accepted.
pub(super) const HANDSHAKE: Duration = Duration::from_secs(30);
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use super::Provider;
use crate::config::outgoing::Outgoing;
use crate::protocol::keepalive;

/// Dial `client` — at its address, as `ws://<address>/` when it carries
/// no scheme — make the handshake as the connector,
/// serve the connection under the peer's identity until it ends, wait
/// five seconds, and dial again: whether the connection ended, the dial
/// failed or the handshake did, and forever. It ends only when the task
/// it runs in is ended. The peer that answers must be the identity
/// `client` names, and no other. The provider
/// announces itself to no peer, and serves the peer's
/// `daemons::announce` as it does for a peer that dialled it, listing
/// it with the address dialled here. The socket has transport keepalive on — see
/// [`keepalive`] — so a peer that vanished without a close is a
/// connection that ends, and a dial again.
pub async fn dial(provider: Arc<Provider>, client: Outgoing) {
    let url = if client.address.contains("://") { client.address.clone() } else { format!("ws://{}/", client.address) };
    loop {
        // Neither the connect nor the handshake waits for ever: a peer
        // that takes the socket and says nothing is a dial that
        // failed, and the next is made. A connection held is never
        // timed.
        match timeout(CONNECT, keepalive::connect(url.as_str())).await {
            Ok(Ok(socket)) => {
                let address = peer_ip(&socket);
                match timeout(HANDSHAKE, handshake::connect(Connection::Outgoing(socket), &provider.authenticator.dialling(&client.identity), address)).await {
                    Ok(Ok(Handshook { connection, peer })) => provider.held(connection, peer, address, Some(client.address.clone())).await,
                    Ok(Err(error)) => provider.log.say(format!("{address} dialled at {}, and its handshake was not taken: {error}", client.address)),
                    Err(_) => provider.log.say(format!("{address} dialled at {}, and its handshake did not finish within {} seconds", client.address, HANDSHAKE.as_secs())),
                }
            }
            Ok(Err(error)) => provider.log.say(format!("{} could not be dialled: {error}", client.address)),
            Err(_) => provider.log.say(format!("{} did not answer within {} seconds", client.address, CONNECT.as_secs())),
        }
        // Five seconds: long enough that a peer that is down is not
        // hammered, short enough that one that is back is served soon.
        sleep(Duration::from_secs(5)).await;
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
