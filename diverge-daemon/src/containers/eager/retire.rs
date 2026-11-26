//! An eager tool retired: its keeping ended.

use std::sync::Arc;

use crate::containers::lifecycle::End;
use crate::containers::{ToolKey, stop};
use crate::daemon::{Daemon, Kind};

/// Retire the eager tool: its state taken out of the live set and its
/// supervisor stilled first, so that nothing starts it again; its
/// run, if one is up, stopped and taken down; and
/// the clock stilled last, since the clock is what calls this. Its
/// row stays — a dependency tool's to go with the row's delete, a
/// tool on record's for its life — and the next attach or connect
/// that holds it keeps it up again. An eager tool not kept is nothing to
/// do.
pub async fn retire(daemon: &Arc<Daemon>, key: ToolKey) {
    let state = daemon.live.remove_eager_state(key).await;
    if let Some(state) = &state {
        if let Some(supervisor) = state.supervisor.lock().await.take() {
            supervisor.abort();
        }
    }
    if let Some(run) = daemon.live.tool_run(key).await {
        run.end_as(End::Expected).await;
        run.handle.stop().await;
        run.handle.wait().await;
        stop::ended_tool(daemon, run).await;
    }
    daemon.live.changed(Kind::DependencyTools);
    daemon.live.changed(Kind::StandaloneTools);
    if let Some(state) = state {
        if let Some(clock) = state.clock.lock().await.take() {
            clock.abort();
        }
    }
}
