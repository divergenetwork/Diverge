//! The agent container's loop, kept: the agent's own server dialled,
//! the queue held, the conversation relayed.
//!
//! An agent container's entrypoint is an HTTP server on the loopback,
//! at the port [`diverge_sdk::container_proxy::inside::port()`] names, and the
//! loop's two calls are what this module makes of it: `/run`
//! whenever messages wait and no loop runs; `/enqueue` with every
//! message waiting, joined in order, while one does — onto
//! the program's own queue, answered at once. The call every
//! container answers — `/register` — is [`program`](crate::program)'s;
//! what the arguments may be is the image's `/.diverge/DIVERGE.md`, which
//! [`info`](crate::info) reads from the container and asks of no program.
//! The queue itself is
//! the proxy's, held by one [`driver()`] task that never
//! awaits the agent's server directly, so nothing the server asks of
//! the queue can wait on a call the loop holds open. The server
//! enqueues messages a batch at a time — one enqueue channel, one
//! batch, its messages in order, its fate every message's alike — and
//! a batch is `delivered` once the program has it — the `/run` that
//! carried it answered `2xx`, or the `/enqueue` `204` — and never
//! waits for a turn; nothing withdraws one. The driver keeps the
//! order, which is the order the batches reached it and each batch's
//! own: a new batch goes to the back of the queue, one call carrying
//! messages is on the wire at a time, and batches that come back —
//! the run closed under them, `409` — go back to the front, ahead of
//! anything that reached the driver after them, and start the next
//! loop, joined.
//!
//! A batch is chat messages, or one structured decision, and the two
//! are handed over differently. Every enqueue carries an `index`, and
//! the driver queues the batches in the order of their indexes,
//! whatever order they reach it in. Chat batches at the head of the
//! queue go as above. A structured decision at the head waits until
//! the program has no run open, starts a `/run` of its own, alone,
//! and nothing — chat or decision — is handed over until that run has
//! ended. So `chat, chat, decision, decision, chat, chat` is: both
//! chats, joined; the first decision once that loop has ended; the
//! second once the first's run has; both chats, joined, once the
//! second's has.
//!
//! What the loop says rides the begin scope's main stream, chunk by
//! chunk, verbatim, between the proxy's own word that the loop began
//! and its word that it ended — `Active` and `Inactive`, one byte
//! each — which
//! the proxy alone writes, since the proxy alone makes the `/run`
//! call whose answer and whose end they are. The server sends the
//! messages it means for one turn as one batch, so `/run` takes every
//! one of them, joined.

mod driver;
mod enqueue;
mod queue;
mod run;

pub use driver::*;
pub use enqueue::*;
pub use queue::*;
