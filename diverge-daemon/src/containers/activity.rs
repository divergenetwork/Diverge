//! Whether an agent is active, followed: its log, its `last_active`
//! and its lists told at every turn; and whether it holds its tools,
//! followed beside it.

use std::sync::Arc;

use super::AgentRun;
use super::lifecycle;
use crate::daemon::{Daemon, Kind};

/// Follow the agent's activity, and its hold on its tools, until the
/// run ends. The rules:
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
/// 5. **Holding its tools is three things, and its own condition.**
///    The agent holds every tool it is served while it is in setup
///    past its registration, while a message is on its way into it,
///    or while it is active — rule 1 of
///    [`Holding`](super::Holding), which is rule 1 of
///    [`idle_tool`](super::idle_tool) — and lets them go when none of
///    the three is so. The hold is taken at every turn to that —
///    [`hold_tools`] — and let go at every turn from it —
///    [`release_tools`]; neither writes anything to the log, and
///    neither is a turn of rule 2. The run's end lets them go as
///    well, whoever ends it.
/// 6. **The first look reconciles.** The setup's hold was taken
///    before this began — at the registration, when the run was not
///    made yet — so the first look takes or lets go by what is so
///    then, whatever was done before it.
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
    let mut setup = run.holding.setup.subscribe();
    let mut pending = run.pending.subscribe();
    let mut was = false;
    // Rule 6: unknown until the first look.
    let mut holding: Option<bool> = None;
    loop {
        let now = *looping.borrow_and_update() || *inflight.borrow_and_update() > 0;
        // Rule 5: read with the two above marked seen, so a change of
        // any of the four after this look wakes the next.
        let wanted = now || *setup.borrow_and_update() > 0 || *pending.borrow_and_update() > 0;
        // Rule 5, before the turn is kept: an agent that turns active
        // holds its tools by the time its log says so.
        if wanted && holding != Some(true) {
            hold_tools(&daemon, &run).await;
        }
        // Rule 2: told at every change — the gate keeps only what moves
        // the log.
        lifecycle::active(daemon.logs.clone(), run.id, Arc::clone(&run.log), run.epoch, now).await;
        // Rules 2 and 3.
        if now != was {
            was = now;
            if let Ok(mut conn) = daemon.store.acquire().await {
                let _ = crate::store::agents::set_active(&mut conn, run.id, chrono::Utc::now()).await;
            }
            daemon.live.changed(Kind::Agents);
        }
        // Rule 5.
        if !wanted && holding != Some(false) {
            release_tools(&daemon, &run).await;
        }
        holding = Some(wanted);
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
            changed = setup.changed() => {
                if changed.is_err() {
                    return;
                }
            }
            changed = pending.changed() => {
                if changed.is_err() {
                    return;
                }
            }
        }
    }
}

/// The agent holds its tools from now: one hold on every tool it is
/// served, as of now, where it has none — rule 3 of
/// [`Holding`](super::Holding). Whoever asks has made one of the three
/// things that hold so first — entered `pending`, or the setup, or
/// seen the agent active — so a release that crosses this leaves the
/// tools held: rule 5 there.
pub async fn hold_tools(daemon: &Daemon, run: &AgentRun) {
    run.holding.hold(daemon, &run.served).await;
}

/// The agent holds its tools no more, where that is still so when it
/// is looked at under the hold's lock: every hold [`hold_tools`] took
/// let go, each tool's run used as of now, so that its idle clock
/// counts from here, and each tool's holds forgotten where they hold
/// nothing more. An agent that is in setup, has a message on its way
/// or is active by then keeps every one.
pub async fn release_tools(daemon: &Daemon, run: &AgentRun) {
    run.holding.release(daemon, || run.holds_tools()).await;
}
