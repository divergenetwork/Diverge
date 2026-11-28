//! The idle clock: an unused container stopped.

use std::sync::Arc;
use std::time::Instant;

use tokio::sync::watch;

use super::lifecycle::End;
use super::{AgentRun, ToolRun};
use crate::daemon::Daemon;

/// Run the agent's idle clock until the run ends. The rules:
///
/// 1. **What holds the run.** While any of these is so, the clock does
///    not run:
///    - the agent is active — a loop runs in it, or an MCP exchange
///      its own container asked for is in flight;
///    - a batch of messages is on its way to it (`pending`: held by
///      the agent's outbox pump from the moment it has the run until
///      the batch is answered);
///    - a use is in progress on it — a filetree watch, a download, an
///      upload, a transfer, a command, a terminal, a frontend request,
///      an `agents connect` scope open on it, with or without an
///      exchange in flight
///      — or a dependency tool of its own that mounts its files is up;
///    - its own program has a scope open on its daemon connection,
///      whatever the scope asks, an endless stream included: an agent
///      that wants to go idle ends its scopes. A scope makes the
///      agent neither active nor busy.
/// 2. **The countdown.** Otherwise, `idle_seconds` after the last use
///    with no use since, the run is stopped. The countdown begins
///    only when nothing holds the run, and begins again from the last
///    touch whenever that becomes so; a hold's end is a touch.
/// 3. **Never under a batch.** The clock never stops a run a batch is
///    going to. Its last look is made under the run's gate lock, and
///    the same step marks the gate `ending`: the outbox pump holds
///    `pending` before it asks for that lock — see
///    [`message`](super::message) — so either the look sees the hold
///    and the run goes on, or the pump finds the gate ending, sends
///    the run nothing, and takes its messages to the next run.
///
/// The pump hears the end and does the rest.
pub async fn idle(daemon: Arc<Daemon>, run: Arc<AgentRun>) {
    let mut active = run.loop_active.subscribe();
    let mut inflight = run.inflight.subscribe();
    let mut pending = run.pending.subscribe();
    let mut files = run.files.subscribe();
    let mut scopes = run.scopes.subscribe();
    let mut touched = run.touched.subscribe();
    loop {
        // Rule 1.
        if *active.borrow() || *inflight.borrow() > 0 || *pending.borrow() > 0 || *files.borrow() > 0 || *scopes.borrow() > 0 {
            tokio::select! {
                changed = active.changed() => {
                    if changed.is_err() {
                        return;
                    }
                }
                changed = inflight.changed() => {
                    if changed.is_err() {
                        return;
                    }
                }
                changed = pending.changed() => {
                    if changed.is_err() {
                        return;
                    }
                }
                changed = files.changed() => {
                    if changed.is_err() {
                        return;
                    }
                }
                changed = scopes.changed() => {
                    if changed.is_err() {
                        return;
                    }
                }
            }
            continue;
        }
        // Rule 2.
        let last = *touched.borrow_and_update();
        let deadline = last + daemon.idle;
        tokio::select! {
            () = tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)) => {
                // Rule 3.
                let mut gate = run.gate.lock().await;
                if !*active.borrow() && *inflight.borrow() == 0 && *pending.borrow() == 0 && *files.borrow() == 0 && *scopes.borrow() == 0 && *touched.borrow() == last {
                    gate.ending = true;
                    drop(gate);
                    run.end_as(End::Expected).await;
                    let _ = run.handle.cancel().await;
                    return;
                }
            }
            changed = touched.changed() => {
                if changed.is_err() {
                    return;
                }
            }
            changed = active.changed() => {
                if changed.is_err() {
                    return;
                }
            }
            changed = inflight.changed() => {
                if changed.is_err() {
                    return;
                }
            }
            changed = pending.changed() => {
                if changed.is_err() {
                    return;
                }
            }
            changed = files.changed() => {
                if changed.is_err() {
                    return;
                }
            }
            changed = scopes.changed() => {
                if changed.is_err() {
                    return;
                }
            }
        }
    }
}

/// Run a tool's idle clock until the run ends. Every tool's run has
/// this one clock — a standalone tool's, a connected tool's and a
/// dependency tool's, lazy or eager, of either scope — and nothing
/// else undeploys a tool for want of use.
///
/// A HOLD stops the clock. The tool is undeployed only when nothing
/// holds it and `idle_seconds` have passed since the last hold ended
/// and since its last use. The holds, each of every kind of tool but
/// where it says otherwise:
///
/// 1. **An agent the tool is attached to is active.** Attached means,
///    for a standalone or a connected tool, attached to the agent or
///    to its template; for a dependency tool, declared by the agent's
///    latest registration. An agent that is deployed and not active
///    holds nothing. The agent's turn to active also deploys the tool:
///    a loop's beginning starts every tool the agent is served.
/// 2. **An MCP exchange in flight into it**: a tool call, a resource
///    read, a tools list or a resources list, from an agent's
///    container, as an agent, or on a connect scope. An exchange
///    deploys the tool it is put to.
/// 3. **An open `standalone-tools connect` scope**, for as long as it
///    is open, with or without an exchange in flight. Opening one
///    deploys the tool.
/// 4. **A file tree watch, a download, an upload or a transfer in
///    progress** on it. Each deploys the tool.
/// 5. **A command in progress in it, or a terminal open in it.** Each
///    deploys the tool.
/// 6. **A frontend request in progress** to it. It deploys the tool.
/// 7. **A scope the tool's own program has open on its daemon
///    connection**, whatever the request, an endless stream included.
///    It deploys nothing: the tool is up to open it.
/// 8. **Eager only: any agent attached to it, running or not.** The
///    attachment deploys the tool, and while it holds, a run that
///    ends is started again by the tool's supervisor.
///
/// None of these makes a tool active: a tool is active only while a
/// tool call or a resource read it was asked is unanswered. A hold
/// that ends starts the countdown from that moment, however long ago
/// the tool was last used; a use that holds nothing — the proxy's
/// word that an exchange began or ended, a frame on the tool's daemon
/// connection — starts it over. When it runs out the run is ended as
/// asked for: the container stopped, or the connection to the other
/// daemon let go, and the waiter hears the end and does the rest.
pub async fn idle_tool(daemon: Arc<Daemon>, run: Arc<ToolRun>) {
    let mut holds = Watched::of(&run);
    let mut touched = run.touched.subscribe();
    // When the last hold ended: the run's start, until one has.
    let mut released = Instant::now();
    loop {
        if holds.any() {
            if !holds.changed().await {
                return;
            }
            released = Instant::now();
            continue;
        }
        let last = *touched.borrow_and_update();
        let deadline = last.max(released) + daemon.idle;
        tokio::select! {
            () = tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)) => {
                if !holds.any() && *touched.borrow() == last {
                    run.end_as(End::Expected).await;
                    run.handle.stop().await;
                    return;
                }
            }
            changed = touched.changed() => {
                if changed.is_err() {
                    return;
                }
            }
            alive = holds.changed() => {
                if !alive {
                    return;
                }
            }
        }
    }
}

/// Every hold on one tool's run, watched: what [`idle_tool`] waits on.
struct Watched {
    /// Rule 3: whether a connect scope is open on it.
    connected: watch::Receiver<bool>,
    /// Rule 2: the exchanges in flight into it.
    inflight: watch::Receiver<usize>,
    /// Rules 4 to 6: the uses in progress on it.
    files: watch::Receiver<usize>,
    /// Rule 1: the active agents it is served to.
    agents: watch::Receiver<usize>,
    /// Rule 7: its own open daemon scopes.
    scopes: watch::Receiver<usize>,
    /// Rule 8: whether an agent is attached to it, an eager tool.
    attached: watch::Receiver<bool>,
}

impl Watched {
    /// The run's holds, from now on.
    fn of(run: &ToolRun) -> Watched {
        Watched {
            connected: run.held.subscribe(),
            inflight: run.inflight.subscribe(),
            files: run.files.subscribe(),
            agents: run.holds.agents.subscribe(),
            scopes: run.scopes.subscribe(),
            attached: run.holds.attached.subscribe(),
        }
    }

    /// Whether anything holds the run now.
    fn any(&self) -> bool {
        *self.connected.borrow() || *self.inflight.borrow() > 0 || *self.files.borrow() > 0 || *self.agents.borrow() > 0 || *self.scopes.borrow() > 0 || *self.attached.borrow()
    }

    /// Wait for any hold to change. `false` once one can change no
    /// more, which is the run gone.
    async fn changed(&mut self) -> bool {
        tokio::select! {
            changed = self.connected.changed() => changed.is_ok(),
            changed = self.inflight.changed() => changed.is_ok(),
            changed = self.files.changed() => changed.is_ok(),
            changed = self.agents.changed() => changed.is_ok(),
            changed = self.scopes.changed() => changed.is_ok(),
            changed = self.attached.changed() => changed.is_ok(),
        }
    }
}
