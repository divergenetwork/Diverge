//! The supervisor: an eager tool's container run again while an agent
//! is attached to it.

use std::sync::Arc;
use std::time::Duration;

use diverge_sdk::daemon::lifecycle::Cause;
use tokio::sync::watch;

use super::{Outcome, State};
use crate::containers::lifecycle::End;
use crate::containers::{ToolKey, ToolRun, stop};
use crate::daemon::Daemon;
use crate::store::{DependencyToolId, ToolId, dependency_tools, tools};

/// How long a supervisor waits after a start that failed before it
/// tries again: long enough not to hammer a provider that is down,
/// short enough that an eager tool is back soon after one comes up.
const RETRY: Duration = Duration::from_secs(5);

/// Keep the eager tool's container running for as long as the tool is
/// kept, which is for as long as an agent is attached to it. The
/// rules:
///
/// 1. **Started while attached.** While the tool's
///    [`Holds`](crate::containers::Holds) say an agent is attached,
///    its container is started — on the first candidate provider that
///    runs it, or found running already — and waited on to its end,
///    and started again. While they say none is, the supervisor
///    waits: the run, if one is up, is its idle clock's.
/// 2. **The first start is the keep's.** It is deployed by what kept
///    the tool — the state's `first` — and every start after a run
///    came up is a [`Cause::Restart`].
/// 3. **A start that fails is tried again** after `RETRY`.
/// 4. **Every start's outcome is said** on `outcomes` — up once the
///    run is live, or why it failed — which is what a registration
///    that declared the eager tool waits on.
/// 5. **A row that is gone is an eager tool deleted**, and the
///    supervisor ends.
///
/// The run's end is taken down here as well as by its waiter, so that
/// the next start does not find the old run still live. Nothing here
/// undeploys a tool: that is its run's idle clock alone, which the
/// attachment holds.
pub async fn supervise(daemon: Arc<Daemon>, key: ToolKey, state: Arc<State>, outcomes: watch::Sender<Option<Outcome>>) {
    let mut attached = daemon.live.holds(key).await.attached.subscribe();
    // Rule 2.
    let mut cause = state.first;
    loop {
        // Rule 1.
        if !*attached.borrow_and_update() {
            if attached.changed().await.is_err() {
                return;
            }
            continue;
        }
        let started = match key {
            ToolKey::Record(id) => record(&daemon, id, cause).await,
            // A connected tool is never an eager tool here: nothing to keep.
            ToolKey::Connected(_) => Started::Gone,
            ToolKey::Dependency(id) => dependency(&daemon, id, cause).await,
        };
        match started {
            // Rule 5.
            Started::Gone => return,
            Started::Run(run) => {
                // Rule 4.
                outcomes.send_replace(Some(Outcome::Up));
                cause = Cause::Restart;
                let ended = run.handle.wait().await;
                run.end_as(End::Unexpected(ended.reason())).await;
                stop::ended_tool(&daemon, run).await;
            }
            // Rule 3.
            Started::Failed(reason) => {
                outcomes.send_replace(Some(Outcome::Failed(reason)));
                tokio::time::sleep(RETRY).await;
            }
        }
    }
}

/// One start's outcome, as the supervisor acts on it.
enum Started {
    /// The run, up, to be waited on.
    Run(Arc<ToolRun>),
    /// The row is gone: the eager tool is deleted.
    Gone,
    /// The store could not be read, or the container would not start,
    /// in its words: tried again after `RETRY`.
    Failed(String),
}

/// One start of a tool on record's container.
async fn record(daemon: &Arc<Daemon>, id: ToolId, cause: Cause) -> Started {
    let tool = match daemon.store.acquire().await {
        Ok(mut conn) => tools::by_id(&mut conn, id, false).await,
        Err(error) => Err(error),
    };
    match tool {
        Ok(Some(tool)) => match crate::containers::start::tool(daemon, &tool, cause).await {
            Ok(run) => Started::Run(run),
            Err(error) => Started::Failed(error.to_string()),
        },
        Ok(None) => Started::Gone,
        Err(error) => Started::Failed(error.to_string()),
    }
}

/// One start of a dependency eager tool's container.
async fn dependency(daemon: &Arc<Daemon>, id: DependencyToolId, cause: Cause) -> Started {
    let tool = match daemon.store.acquire().await {
        Ok(mut conn) => dependency_tools::by_id(&mut conn, id, false).await,
        Err(error) => Err(error),
    };
    match tool {
        Ok(Some(tool)) => match crate::containers::deploy::tool(daemon, &tool, None, cause).await {
            Ok(run) => Started::Run(run),
            Err(error) => Started::Failed(error.to_string()),
        },
        Ok(None) => Started::Gone,
        Err(error) => Started::Failed(error.to_string()),
    }
}
