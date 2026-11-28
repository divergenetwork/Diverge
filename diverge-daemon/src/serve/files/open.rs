//! A container opened for one file operation.

use std::sync::Arc;

use diverge_sdk::daemon::lifecycle::Cause;

use crate::containers::{self, Opened, StartError};
use crate::daemon::Daemon;
use crate::store::DependencyToolId;
use crate::store::agents::Agent;
use crate::store::tools::Tool;

/// The agent's container, started if it was not up — `cause` being
/// what started it — with the use
/// held on it for as long as the [`Opened`] lives: the idle clock stops
/// it `idle_seconds` after the use and every other one are over, which
/// is what "stopped when it finishes" means under the idle rule.
pub async fn open_agent(daemon: &Arc<Daemon>, agent: &Agent, cause: Cause) -> Result<Opened, String> {
    containers::agent(daemon, agent, cause)
        .await
        .map(|run| {
            let using = run.file_use();
            Opened::Agent(run, using)
        })
        .map_err(|error| error.to_string())
}

/// The tool's container, started if it was not up — `cause` being what
/// started it — with the use
/// held on it for as long as the [`Opened`] lives — a hold, not work: it
/// makes the tool neither active nor busy, and the tool's idle clock
/// stops it `idle_seconds` after this and every other hold are over.
/// A connected tool is connected to the
/// same way, and every file operation on it then answers the handle's
/// error: its files are the other daemon's.
pub async fn open_tool(daemon: &Arc<Daemon>, tool: &Tool, cause: Cause) -> Result<Opened, String> {
    let run = containers::use_tool(daemon, tool, cause).await.map_err(|error| error.to_string())?;
    let using = run.file_use();
    Ok(Opened::Tool(run, Some(using)))
}

/// The dependency tool's container, deployed if it was not up as
/// [`use_dependency`](containers::use_dependency) deploys one —
/// `cause` being what deployed it — with the use held on it for as
/// long as the [`Opened`] lives: a hold on the tool's own run, as a
/// standalone tool's is. A tool that is on record no more is
/// [`StartError::Gone`], which a request answers as not found.
pub async fn open_dependency(daemon: &Arc<Daemon>, id: DependencyToolId, cause: Cause) -> Result<Opened, StartError> {
    let run = containers::use_dependency(daemon, id, cause).await?;
    let using = run.file_use();
    Ok(Opened::Tool(run, Some(using)))
}
