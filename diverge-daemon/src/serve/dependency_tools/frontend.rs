//! One HTTP request to the frontend of a dependency tool's container.

use std::sync::Arc;

use diverge_sdk::daemon::endpoints::dependency_tools::frontend::client::request;
use diverge_sdk::daemon::endpoints::dependency_tools::frontend::server::response::Frame;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;

use super::door;
use crate::containers::Door;
use crate::daemon::Daemon;
use crate::judge::Who;

/// Send the request, serve its answer and its body, and finish the
/// scope: [`door::handle`], through the tool's frontend.
pub async fn handle(scope: ScopeHandle, frame: request::Frame, who: Who, daemon: &Arc<Daemon>) {
    door::handle::<Frame>(scope, frame.dependency, frame.head, who, daemon, Door::Frontend).await;
}
