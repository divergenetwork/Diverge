//! The daemon stopping: every supervisor stilled.

use crate::daemon::Daemon;

/// Still every eager tool's supervisor, so that no run stopped at the
/// daemon's end is started again. The runs themselves are stopped
/// with every other run; the eager tools come back at the next boot.
pub async fn stop_all(daemon: &Daemon) {
    for state in daemon.live.eager_states().await {
        if let Some(supervisor) = state.supervisor.lock().await.take() {
            supervisor.abort();
        }
    }
}
