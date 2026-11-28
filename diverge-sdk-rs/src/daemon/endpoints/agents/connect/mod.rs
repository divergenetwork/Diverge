//! MCP as an agent: the scope a client runs an agent's MCP exchanges
//! through, as though the agent's own container ran them.
//!
//! A client names an agent; the daemon judges it, deploys the agent's
//! container if it is down, and answers exactly
//! one response, that it is connected; then the scope stays open, and
//! the client opens channels on it: one per MCP exchange — the five of
//! [`shared::mcp`](crate::shared::mcp), each answered from the tools
//! served to the agent, the four that answer once with one channel
//! response and the finish, the notifications with one per
//! notification until the scope ends — and the disconnect, which ends
//! the scope. Or the daemon answers that no agent is the one named,
//! forbidden, or that it failed — the agent's container would not
//! start among the failures — and finishes.
//!
//! # The scope holds the agent, and makes it nothing
//!
//! The rules:
//!
//! 1. **Deployed, and held.** Opening the scope deploys the agent's
//!    container if it is down — the ordinary start, which deploys
//!    every tool the agent is served and fails where one cannot be —
//!    and the scope holds the container up for as long as it is open,
//!    with or without an exchange in flight. The idle countdown starts
//!    when the scope closes. A container that will not start is the
//!    connect's `error`.
//! 2. **Never active.** Nothing on the scope makes the agent active,
//!    and nothing on it keeps the agent active: no `active` and no
//!    `inactive` is kept in the agent's log for it. Every exchange is
//!    answered all the same, exactly as the same exchange from the
//!    agent's container is: the agent's tools, by the names the agent
//!    sees them by; each request stamped as the agent's under `_meta`,
//!    on the agent's own session with each tool; each call and read
//!    logged in the tool's log as the agent's. Who opened the scope is
//!    not told to any tool.
//! 3. **The tool an exchange reaches is held.** A list, a call or a
//!    read holds the tool it reaches while it is in flight, and
//!    deploys that tool if it is down.
//! 4. **Three ends.** The scope ends when the client disconnects, when
//!    its connection ends, or when the agent's container ends, however
//!    it ends; a container that ended is not started again under the
//!    scope.
//! 5. **What makes an agent active.** Exactly two things: a loop
//!    running in it, which a message starts, and an MCP exchange in
//!    flight that its own container asked for. A connect is neither.
//!
//! Split by who SENDS, as everywhere else. A client asks — so the
//! question and the channels are in [`client`] — and the daemon
//! answers, so the answers are in [`server`]. Neither side holds both
//! halves of the exchange.

pub mod client;
pub mod server;
