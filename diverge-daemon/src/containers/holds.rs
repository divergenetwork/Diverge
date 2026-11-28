//! What holds a tool up from outside its run.

use std::sync::Arc;

use tokio::sync::watch;

use super::Inflight;

/// The holds on one tool that are not any one run's to count: kept by
/// the tool's key in the live set, so that one can be taken while the
/// tool has no run, and the run, when it is made, reads the same
/// counts. The rules:
///
/// 1. **One per tool.** [`Live::holds`](crate::daemon::Live::holds)
///    hands out the one kept under the key, made at the first ask.
///    A run takes it at its start and keeps it for its life.
/// 2. **`agents`** counts the agents the tool is served to that hold
///    their tools now — each in setup past its registration, with a
///    message on its way, or active: entered when an agent comes to
///    hold its tools, left when it holds them no more and at its
///    run's end — see [`Holding`](super::Holding).
/// 3. **`attached`** is whether any agent is attached to the tool,
///    running or not, said only of an eager tool — see
///    [`eager::held`](super::eager::held). A lazy tool's is never
///    `true`.
/// 4. **Forgotten when it holds nothing.** An entry leaves the live
///    set once nothing is counted in it, nothing is attached, no run
///    is live or starting under its key and no eager tool is kept
///    under it — see
///    [`Live::release_holds`](crate::daemon::Live::release_holds).
///
/// The scopes a tool's own program has open on the daemon are not
/// here: they are one run's — opened by that run's container, and
/// over with it — and are counted on the run itself.
#[derive(Debug)]
pub struct Holds {
    /// The agents the tool is served to that hold their tools now:
    /// rule 2.
    pub agents: Arc<Inflight>,
    /// Whether any agent is attached to an eager tool: rule 3.
    pub attached: watch::Sender<bool>,
}

impl Holds {
    /// Nothing held.
    pub fn new() -> Self {
        Holds { agents: Arc::new(Inflight::new()), attached: watch::channel(false).0 }
    }

    /// Whether anything is held here now.
    pub fn any(&self) -> bool {
        self.agents.is_busy() || *self.attached.borrow()
    }
}

impl Default for Holds {
    fn default() -> Self {
        Holds::new()
    }
}
