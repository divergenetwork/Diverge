//! What a container's program declared it may ask of the daemon, kept
//! on its row once its run is up.

use diverge_sdk::shared::permission::Grant;

use crate::daemon::{Daemon, Kind};
use crate::judge::{Container, ToolRef};
use crate::store::{self, agents, dependency_tools, tools};

/// Keep `permissions` — what the run's registration declared, empty
/// for a program that declared none — on the container's row, as what
/// it last declared, and tell its lists. Written at every start, so
/// the row follows the image.
pub async fn record(daemon: &Daemon, container: Container, permissions: &[Grant]) -> Result<(), store::Error> {
    let mut conn = daemon.store.acquire().await?;
    match container {
        Container::Agent(id) => {
            agents::set_permissions(&mut conn, id, permissions).await?;
            daemon.live.changed(Kind::Agents);
        }
        Container::Tool(ToolRef::Standalone(id)) => {
            tools::set_permissions(&mut conn, id, permissions).await?;
            daemon.live.changed(Kind::StandaloneTools);
        }
        Container::Tool(ToolRef::Dependency(id)) => {
            dependency_tools::set_permissions(&mut conn, id, permissions).await?;
            daemon.live.changed(Kind::DependencyTools);
        }
    }
    Ok(())
}
