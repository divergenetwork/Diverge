//! The daemon stopping: every supervisor and every clock stilled.

use crate::daemon::Daemon;

/// Still every eager tool's supervisor and clock, so that no run
/// stopped at the daemon's end is started again and no eager tool is
/// retired on the way down. The runs themselves are stopped with
/// every other run; the eager tools come back at the next boot.
pub async fn stop_all(daemon: &Daemon) {
    for state in daemon.live.eager_states().await {
        if let Some(supervisor) = state.supervisor.lock().await.take() {
            supervisor.abort();
        }
        if let Some(clock) = state.clock.lock().await.take() {
            clock.abort();
        }
    }
}
