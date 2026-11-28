//! An eager tool kept, and kept no more.

use std::sync::Arc;

use diverge_sdk::daemon::lifecycle::Cause;
use tokio::sync::watch;

use super::{State, supervise};
use crate::containers::ToolKey;
use crate::daemon::Daemon;

/// Keep the eager tool: its [`State`] made if it had none, and its
/// supervisor started on a task of its own, saying each start on the
/// state, the first deployed by `first`. One kept already is left as
/// it is. Two keeps racing make one state: the first in wins, and the
/// second starts nothing. Asked only by [`held`](super::held), which
/// keeps a tool exactly while an agent is attached to it.
pub(super) async fn keep(daemon: &Arc<Daemon>, key: ToolKey, first: Cause) {
    let (outcomes, started) = watch::channel(None);
    let (state, made) = daemon.live.keep_eager_state(key, Arc::new(State::new(first, started))).await;
    if !made {
        return;
    }
    let supervisor = tokio::spawn(supervise(Arc::clone(daemon), key, Arc::clone(&state), outcomes));
    *state.supervisor.lock().await = Some(supervisor.abort_handle());
}

/// Keep the eager tool no more: its state taken out of the live set
/// and its supervisor stilled, so that nothing starts it again. Its
/// run, if one is up, is left to its idle clock. One not kept is
/// nothing to do.
pub(super) async fn unkeep(daemon: &Daemon, key: ToolKey) {
    if let Some(state) = daemon.live.remove_eager_state(key).await
        && let Some(supervisor) = state.supervisor.lock().await.take()
    {
        supervisor.abort();
    }
}
