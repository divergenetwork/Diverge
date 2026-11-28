//! An API request: one HTTP request to the program's server, beneath
//! `/api`.

use diverge_sdk::container_proxy::outside::endpoints::api::client::request;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;

use super::door::{Door, door};

/// Serve one API request: [`door`], through the program's API.
pub async fn api(scope: ScopeHandle, frame: request::Frame) {
    door(scope, frame.0, Door::Api).await;
}
