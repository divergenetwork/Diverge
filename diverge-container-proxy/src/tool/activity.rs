//! The tool container's activity: the MCP exchanges in flight.

use diverge_sdk::container_proxy::outside::endpoints::tools::begin::server::response::{ACTIVE, INACTIVE};
use diverge_sdk::wire::server::scope_handle::ScopeHandle;
use tokio::sync::Mutex;

/// How many MCP exchanges are in flight.
///
/// An exchange entering with none in flight is the container's
/// `Active`; the last leaving is its `Inactive`. The count is one
/// lock, held across the word written for a change of it, so the two
/// words alternate on the wire as the count does.
pub struct Activity {
    inflight: Mutex<usize>,
}

impl Activity {
    pub fn new() -> Self {
        Activity { inflight: Mutex::new(0) }
    }

    /// One exchange begins: the count up, and `Active` on the main
    /// stream when it went from none to one, under the lock.
    pub async fn enter(&self, scope: &ScopeHandle) {
        let mut inflight = self.inflight.lock().await;
        *inflight += 1;
        if *inflight == 1 {
            scope.send_response(&[ACTIVE]).await;
        }
    }

    /// One exchange is over, its reply written: the count down, and
    /// `Inactive` on the main stream when it went from one to none,
    /// under the lock, so no exchange enters between the count and the
    /// word.
    pub async fn leave(&self, scope: &ScopeHandle) {
        let mut inflight = self.inflight.lock().await;
        *inflight -= 1;
        if *inflight == 0 {
            scope.send_response(&[INACTIVE]).await;
        }
    }
}
