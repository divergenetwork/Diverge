//! The HTTP stop: `POST /stop`, from loopback only.
//!
//! The viewer is ended as the daemon and the provider are: by a
//! request put to a port of its own from this host. The rules:
//!
//! - The port is the settings' `port`, bound on `127.0.0.1` and no
//!   other interface, before the window is made: a port that cannot
//!   be listened on stops the start.
//! - Only a loopback peer is heard. Any other is answered
//!   `403 Forbidden` with nothing done; the bind already keeps them
//!   out, and the check is kept as the daemon keeps its own.
//! - A loopback peer is answered `204 No Content`, and the app is
//!   told to exit. The exit closes the window, ends the connection
//!   to the daemon, and lets the root's lock go, last, with the
//!   process: the lock free is the stop done, which is what the
//!   caller waits for.
//! - Nothing else is served: every other path and method is `404` or
//!   `405`.

use std::net::SocketAddr;

use axum::Router;
use axum::extract::State;
use axum::extract::connect_info::ConnectInfo;
use axum::http::StatusCode;

/// The stop served on `listener` for the app's life.
pub fn serve(app: tauri::AppHandle, listener: tokio::net::TcpListener) {
    let router = Router::new().route("/stop", axum::routing::post(stop)).with_state(app);
    tauri::async_runtime::spawn(async move {
        let _ = axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>()).await;
    });
}

/// `POST /stop`: see the module's rules.
async fn stop(State(app): State<tauri::AppHandle>, ConnectInfo(peer): ConnectInfo<SocketAddr>) -> StatusCode {
    if !peer.ip().to_canonical().is_loopback() {
        return StatusCode::FORBIDDEN;
    }
    app.exit(0);
    StatusCode::NO_CONTENT
}
