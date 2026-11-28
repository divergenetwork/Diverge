//! An agent's hold on every tool it is served, and its setup.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use super::mcp::Served;
use super::{Guard, Inflight, ToolKey};
use crate::daemon::Daemon;

/// One agent's hold on the tools it is served: made with the start
/// that makes its run, shared by its answerer — which the
/// registration reaches before there is a run — and the run. The
/// rules:
///
/// 1. **When it holds.** The agent holds every tool it is served
///    while any of these is so, and at no other time:
///    - it is in setup past its registration (rule 2);
///    - a message is on its way into it — the run's `pending`;
///    - it is active.
///
///    This is rule 1 of [`idle_tool`](super::idle_tool), from the
///    agent's side. It makes no tool active, and it is not what makes
///    the agent active.
/// 2. **Setup.** An agent's setup is its start, from its container
///    coming up to the start being over. The hold begins partway —
///    [`begin`](Self::begin) — at its registration, once what its
///    program declared is on record and so the tools it is served are
///    known: at the registration itself where the program declared a
///    dependency, and where it declared nothing, which is registered
///    for nothing, as soon as its container's id is out. `setup` is
///    counted in from there until the start is over —
///    [`end`](Self::end) — or has failed — [`forget`](Self::forget).
/// 3. **One hold per tool, by its key.** A hold is on the tool,
///    counted whether or not the tool has a run — see
///    [`Holds`](super::Holds) — and taken once: [`hold`](Self::hold)
///    takes what is missing and leaves the rest.
/// 4. **In step with the served set.** `held` is locked before the
///    served set is read, and whoever serves the agent a tool or
///    takes one away locks it first as well — see
///    [`attached`](super::attached) and [`detached`](super::detached)
///    — so a tool that comes or goes while the agent holds is held or
///    let go with it.
/// 5. **Let go only when nothing of rule 1 is so.**
///    [`release`](Self::release) asks again under `held`'s lock, and
///    whoever makes one of the three so counts it in before it asks
///    for the hold: so a release and a hold that cross leave the tools
///    held.
#[derive(Debug)]
pub struct Holding {
    /// The holds, one guard per tool: `Some` while the agent holds
    /// its tools, `None` while it does not. Taken before the served
    /// set by whoever takes both: rule 4.
    pub held: Mutex<Option<HashMap<ToolKey, Guard>>>,
    /// Whether the agent is in setup past its registration: counted
    /// in by [`begin`](Self::begin), once. What its
    /// [`activity`](super::activity) reads, beside the two others of
    /// rule 1.
    pub setup: Arc<Inflight>,
    /// The setup's own count, from [`begin`](Self::begin) until
    /// [`end`](Self::end) hands it out.
    entered: Mutex<Option<Guard>>,
}

impl Holding {
    /// Nothing held, and no setup begun.
    pub fn new() -> Arc<Holding> {
        Arc::new(Holding { held: Mutex::new(None), setup: Arc::new(Inflight::new()), entered: Mutex::new(None) })
    }

    /// Rule 2: the setup is past its registration from now — counted
    /// in, once, however many times this is asked — and the agent
    /// holds every tool it is served, with every one of `declared`: the
    /// dependency tools this registration put on record, which are
    /// served only once they are deployed.
    pub async fn begin(&self, daemon: &Daemon, served: &Mutex<Served>, declared: &[ToolKey]) {
        {
            let mut entered = self.entered.lock().await;
            if entered.is_none() {
                *entered = Some(self.setup.enter());
            }
        }
        let mut held = self.held.lock().await;
        let mut keys = served.lock().await.keys();
        keys.extend_from_slice(declared);
        let held = held.get_or_insert_default();
        for key in keys {
            if !held.contains_key(&key) {
                held.insert(key, daemon.live.hold_agent(key).await);
            }
        }
    }

    /// Rule 2: the start is over. The setup's count, for whoever
    /// carries it on — a start made for a message hands it to the
    /// message's own hold, see [`AgentRun::arrival`](super::AgentRun)
    /// — or drops it there and then; `None` where no setup had begun.
    pub async fn end(&self) -> Option<Guard> {
        self.entered.lock().await.take()
    }

    /// Rule 3: one hold on every tool the agent is served, as of now,
    /// where it has none — an agent holding already keeps what it
    /// holds, and takes what is missing.
    pub async fn hold(&self, daemon: &Daemon, served: &Mutex<Served>) {
        let mut held = self.held.lock().await;
        let keys = served.lock().await.keys();
        let held = held.get_or_insert_default();
        for key in keys {
            if !held.contains_key(&key) {
                held.insert(key, daemon.live.hold_agent(key).await);
            }
        }
    }

    /// Rule 5: every hold let go, unless `wanted` says, under the
    /// lock, that the agent holds its tools still. Each tool's run is
    /// used as of now, so that its idle clock counts from here, and
    /// each tool's holds are forgotten where they hold nothing more.
    pub async fn release(&self, daemon: &Daemon, wanted: impl Fn() -> bool) {
        let let_go = {
            let mut held = self.held.lock().await;
            if wanted() {
                return;
            }
            held.take()
        };
        dropped(daemon, let_go).await;
    }

    /// The run ended, or the start failed: the setup counted out and
    /// every hold let go, whatever rule 1 says, as
    /// [`release`](Self::release) lets them go.
    pub async fn forget(&self, daemon: &Daemon) {
        drop(self.entered.lock().await.take());
        let let_go = self.held.lock().await.take();
        dropped(daemon, let_go).await;
    }
}

/// The holds dropped, then each tool touched and its holds forgotten
/// where they hold nothing: the count is out before the entry is
/// looked at.
async fn dropped(daemon: &Daemon, let_go: Option<HashMap<ToolKey, Guard>>) {
    let Some(let_go) = let_go else {
        return;
    };
    let keys: Vec<ToolKey> = let_go.keys().copied().collect();
    drop(let_go);
    for key in keys {
        if let Some(tool) = daemon.live.tool_run(key).await {
            tool.touch();
        }
        daemon.live.release_holds(key).await;
    }
}
