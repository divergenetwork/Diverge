//! An eager tool's start, waited for.

use super::Outcome;
use crate::containers::ToolKey;
use crate::daemon::Daemon;

/// The eager tool up: its run live now, or its supervisor's next start
/// waited for and up. Why it is not, in a sentence: the start's own
/// words, or an eager tool not kept. The last word the supervisor said
/// is marked read before the run is looked for, so a start that came
/// up between the two is seen live — a run is live before its
/// outcome is said — and an outcome said before this ask, a failure
/// that is over, is not read: the next is the start this ask waits
/// on.
pub async fn up(daemon: &Daemon, key: ToolKey) -> Result<(), String> {
    let Some(state) = daemon.live.eager_state(key).await else {
        return Err("the eager tool is not kept".to_string());
    };
    let mut started = state.started.clone();
    started.borrow_and_update();
    if daemon.live.tool_run(key).await.is_some() {
        return Ok(());
    }
    if started.changed().await.is_err() {
        return Err("the eager tool is not kept any more".to_string());
    }
    let outcome = (*started.borrow_and_update()).clone();
    match outcome {
        Some(Outcome::Failed(reason)) => Err(reason),
        _ => Ok(()),
    }
}
