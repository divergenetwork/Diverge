//! A frontend request: one HTTP request to the program's server,
//! beneath `/frontend`.

use diverge_sdk::container_proxy::outside::endpoints::frontend::client::request;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;

use super::door::{Door, door};

/// Serve one frontend request: [`door`], through the program's
/// frontend.
pub async fn frontend(scope: ScopeHandle, frame: request::Frame) {
    door(scope, frame.0, Door::Frontend).await;
}
