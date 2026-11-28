//! Whether an agent is active, followed: its log, its `last_active`
//! and its lists told at every turn, and every tool it is served held
//! while it is.

use std::collections::HashMap;
use std::sync::Arc;

use super::lifecycle;
use super::{AgentRun, ToolKey};
use crate::daemon::{Daemon, Kind};

/// Follow the agent's activity until the run ends. The rules:
///
/// 1. **Active is two things.** The agent is active exactly while a
///    loop runs in it or an MCP exchange its own container asked for
///    is in flight, as [`AgentRun::is_active`] states. An exchange run
///    as it on an `agents connect` scope is neither: it is counted
///    nowhere on the agent's run.
/// 2. **Every turn is kept.** When it becomes active, and when it
///    ceases, the log keeps `active` or `inactive` — only while this
///    run is the one deployed, as
///    [`lifecycle::active`](super::lifecycle::active) gates it — now is
///    kept as its `last_active`, and every open list of agents is told.
/// 3. **Only turns.** A loop beginning while an exchange is in flight,
///    or an exchange beginning in a loop, is no turn, and nothing is
///    kept or told for it.
/// 4. **A time that cannot be kept fails nothing.**
/// 5. **An active agent holds its tools.** At the turn to active the
///    agent takes one hold on every tool it is served —
///    [`hold_tools`] — and at the turn to inactive lets every one
///    go — [`release_tools`]:
///    rule 1 of [`idle_tool`](super::idle_tool). The run's end lets
///    them go as well, whoever ends it.
///
/// The run starts inactive, and is spawned after its `deployed`; its
/// end, however it comes, is told by the stop, which keeps the last
/// `inactive` itself before `undeployed`. An exchange cut short — a
/// scope or a connection under it gone — changes nothing here: the
/// agent is active or not as rule 1 states, and if its run is in fact
/// going down, the stop says so.
pub async fn activity(daemon: Arc<Daemon>, run: Arc<AgentRun>) {
    let mut looping = run.loop_active.subscribe();
    let mut inflight = run.inflight.subscribe();
    let mut was = false;
    loop {
        let now = *looping.borrow_and_update() || *inflight.borrow_and_update() > 0;
        // Rule 2: told at every change — the gate keeps only what moves
        // the log.
        lifecycle::active(daemon.logs.clone(), run.id, Arc::clone(&run.log), run.epoch, now).await;
        // Rules 2 and 3.
        if now != was {
            was = now;
            // Rule 5.
            if now {
                hold_tools(&daemon, &run).await;
            } else {
                release_tools(&daemon, &run).await;
            }
            if let Ok(mut conn) = daemon.store.acquire().await {
                let _ = crate::store::agents::set_active(&mut conn, run.id, chrono::Utc::now()).await;
            }
            daemon.live.changed(Kind::Agents);
        }
        tokio::select! {
            changed = looping.changed() => {
                if changed.is_err() {
                    return;
                }
            }
            changed = inflight.changed() => {
                if changed.is_err() {
                    return;
                }
            }
        }
    }
}

/// The agent turned active: one hold on every tool it is served, as
/// of now, kept on the run until [`release_tools`]. The rules:
///
/// 1. **Every served tool, running or not.** A hold is on the tool,
///    by its key, and is counted whether or not the tool has a run;
///    the loop's beginning starts the runs.
/// 2. **In step with the served set.** `holding` is locked before the
///    served set is read, and whoever serves the agent a tool or
///    takes one away locks it first as well — see
///    [`attached`](super::attached) and [`detached`](super::detached)
///    — so a tool that comes or goes while the agent is active is
///    held or let go with it.
/// 3. **Once.** An agent holding already is left as it is.
pub async fn hold_tools(daemon: &Daemon, run: &AgentRun) {
    let mut holding = run.holding.lock().await;
    if holding.is_some() {
        return;
    }
    let keys = run.served.lock().await.keys();
    let mut held = HashMap::with_capacity(keys.len());
    for key in keys {
        held.insert(key, daemon.live.hold_agent(key).await);
    }
    *holding = Some(held);
}

/// The agent turned inactive, or its run ended: every hold
/// [`hold_tools`] took let go, each tool's run used as of now, so that its idle
/// clock counts from here, and each tool's holds forgotten where they
/// hold nothing more.
pub async fn release_tools(daemon: &Daemon, run: &AgentRun) {
    let Some(held) = run.holding.lock().await.take() else {
        return;
    };
    let keys: Vec<ToolKey> = held.keys().copied().collect();
    drop(held);
    for key in keys {
        if let Some(tool) = daemon.live.tool_run(key).await {
            tool.touch();
        }
        daemon.live.release_holds(key).await;
    }
}
