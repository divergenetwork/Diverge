//! A tool whose record is gone: its keeping ended, its run stopped.

use std::sync::Arc;

use super::unkeep;
use crate::containers::lifecycle::End;
use crate::containers::{ToolKey, stop};
use crate::daemon::{Daemon, Kind};

/// The tool is deleted — a tool on record, or a dependency tool whose
/// row went: if it was an eager tool kept, it is kept no more, its
/// supervisor stilled first so that nothing starts it again; its run,
/// if one is up, is stopped and taken down, whatever held it; and its
/// holds, which now hold nothing that could be served, are forgotten.
/// A tool with neither is nothing to do. This is the one end of a
/// tool's run that is not its idle clock's: a delete does not wait
/// for a tool to go idle.
pub async fn retire(daemon: &Arc<Daemon>, key: ToolKey) {
    unkeep(daemon, key).await;
    if let Some(run) = daemon.live.tool_run(key).await {
        run.end_as(End::Expected).await;
        run.handle.stop().await;
        run.handle.wait().await;
        stop::ended_tool(daemon, run).await;
    }
    daemon.live.holds(key).await.attached.send_replace(false);
    daemon.live.release_holds(key).await;
    daemon.live.changed(Kind::DependencyTools);
    daemon.live.changed(Kind::StandaloneTools);
}
