//! What is live for one eager tool, while it is kept.

use diverge_sdk::daemon::lifecycle::Cause;
use tokio::sync::{Mutex, watch};
use tokio::task::AbortHandle;

use super::Outcome;

/// An eager tool as the daemon keeps it, run or no run, while any
/// agent is attached to it: what its supervisor's last start came to,
/// and the supervisor's task, which runs its container again whenever
/// a run ends. Lives from the attachment that first held the tool to
/// the detach that left it held by none, across every run of its
/// container between. Whether any agent is attached is not here: it
/// is the tool's [`Holds`](crate::containers::Holds), which its
/// supervisor and its run's idle clock both read.
#[derive(Debug)]
pub struct State {
    /// What the supervisor's first start is deployed by: the
    /// attachment, the registration or the boot that kept the tool.
    /// Every start after it is a restart.
    pub first: Cause,
    /// The supervisor's last start, `None` before its first: what a
    /// registration that declared the eager tool waits on for it to be
    /// up or to have failed. The supervisor holds the sender, so the
    /// word ends with it — the eager tool kept no more, or its row
    /// gone.
    pub started: watch::Receiver<Option<Outcome>>,
    /// The supervisor's task, once started.
    pub supervisor: Mutex<Option<AbortHandle>>,
}

impl State {
    /// An eager tool kept, as of now, its supervisor not yet started,
    /// its starts to be told on `started`.
    pub fn new(first: Cause, started: watch::Receiver<Option<Outcome>>) -> State {
        State { first, started, supervisor: Mutex::new(None) }
    }
}
