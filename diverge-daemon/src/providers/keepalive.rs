//! Transport keepalive on every TCP socket the daemon dials a provider
//! on.
//!
//! A provider that vanishes without closing — a laptop asleep, a
//! machine powered off, a cable pulled — leaves a socket that is open
//! on this end and never hears another byte. Without keepalive the
//! daemon would hold that provider connected for as long as it runs,
//! and never dial it again. TCP keepalive is the transport's own
//! answer: after `IDLE` of silence the kernel probes the peer every
//! `INTERVAL`, and a peer that answers none of the probes is a socket
//! the kernel resets, which the read side sees as the connection
//! ending — and the dial dials again. It is not a deadline on
//! anything the protocol does: a connection with nothing to say for a
//! week stays up for a week, so long as the provider is there. The
//! provider turns the same on, on its own end.

use std::time::Duration;

use socket2::{SockRef, TcpKeepalive};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::client::IntoClientRequest as _;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

/// How long a socket is silent before the first probe.
pub const IDLE: Duration = Duration::from_secs(60);

/// How long between one probe and the next, once probing.
pub const INTERVAL: Duration = Duration::from_secs(15);

/// Dial `url` as [`connect_async`](tokio_tungstenite::connect_async)
/// does — the TCP connection to the URL's host and port, `80` when it
/// names none, and the WebSocket upgrade on it — with keepalive turned
/// on between the two. Only `ws://`: the crate speaks no TLS. A socket
/// that refuses the option is used without it.
pub async fn connect(url: &str) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Error> {
    let request = url.into_client_request()?;
    let port = match request.uri().scheme_str() {
        Some("ws") => request.uri().port_u16().unwrap_or(80),
        Some("wss") => return Err(tungstenite::Error::Url(tungstenite::error::UrlError::TlsFeatureNotEnabled)),
        _ => return Err(tungstenite::Error::Url(tungstenite::error::UrlError::UnsupportedUrlScheme)),
    };
    let host = request.uri().host().ok_or(tungstenite::Error::Url(tungstenite::error::UrlError::NoHostName))?;
    let socket = TcpStream::connect(format!("{host}:{port}")).await.map_err(tungstenite::Error::Io)?;
    let _ = SockRef::from(&socket).set_tcp_keepalive(&TcpKeepalive::new().with_time(IDLE).with_interval(INTERVAL));
    let (socket, _) = tokio_tungstenite::client_async(request, MaybeTlsStream::Plain(socket)).await?;
    Ok(socket)
}
