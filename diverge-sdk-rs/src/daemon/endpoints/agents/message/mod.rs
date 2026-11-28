//! Asking an agent something: a chat message, or a structured
//! decision.
//!
//! One request, whose `type` says which of the two things an agent's
//! image may say it does in its `/.diverge/DIVERGE.md`:
//!
//! - `chat` hands the agent a message — MCP content blocks — which
//!   joins its conversation.
//! - `structured_decisions` hands it a prompt and typed questions,
//!   which it answers with numbers.
//!
//! Both go to the agent in the order the daemon admitted them,
//! whichever kind each is. They are handed over differently: a chat
//! message joins a loop that is running, and a structured decision
//! waits for the agent to be idle, runs alone, and is finished before
//! anything after it starts — see
//! [`enqueue`](crate::shared::containers::enqueue).
//!
//! # A message is what makes an agent active
//!
//! The rules:
//!
//! 1. **A message deploys the agent** if its container is down, and
//!    with it every tool the agent is served: the dependency tools
//!    its program declares, of either scope, lazy or eager, the
//!    standalone tools attached to it or to its template, and the
//!    connected tools. All are started together and all are waited
//!    for.
//! 2. **A tool that cannot be deployed fails the message.** The
//!    agent's start fails, naming the tool, and every message waiting
//!    on it is answered with that error.
//! 3. **A message for an agent that is up already** first brings
//!    back any tool it is served that has gone down since; one that
//!    cannot be brought back fails the message, and nothing is
//!    delivered. No loop begins with a tool the agent is served down.
//! 4. **The agent holds its tools** from its registration, while a
//!    message is on its way into it, and while it is active: a tool
//!    it is served is not undeployed for want of use meanwhile.
//! 5. **Active is the loop.** The agent is active while a loop runs
//!    in it — which only a message starts — or an MCP exchange its
//!    own container asked for is in flight, and at no other time.
//!
//! Unwatched, the request is one question and one answer, open until
//! the fate is known: the agent's program has it, or it failed.
//! Watched, the answer is the part of the agent's log that is the
//! request's: the request as the agent echoed it, and everything the
//! log took after, to the end of that run — see the
//! [response](server::response::Frame). The daemon enqueues each
//! request on the provider's wire under a key of its own minting, and
//! nothing takes one back once sent.
//!
//! One grant covers both: `message`, over agents.
//!
//! Split by who SENDS, as everywhere else. A client asks — so the
//! question and the cancel are in [`client`] — and the daemon answers,
//! so the answer is in [`server`].

pub mod client;
pub mod server;
