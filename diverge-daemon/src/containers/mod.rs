//! The work layer: containers running on providers, and everything
//! that runs through them.
//!
//! An agent or a tool is its record; the container is work made from
//! it when it is first used and ended when it has gone unused for
//! `idle_seconds`, or stopped with the daemon. [`agent`] and [`tool`]
//! make the run — the providers it may run on, [`candidates`], tried
//! in order; the mounts served; the [`answerers`] that take the
//! provider's asks on the run scope; the `run` opened — and
//! [`stop_agent`], [`ended_agent`], [`ended_tool`] and [`stop_all`]
//! end it; [`pump`] reads the agent's conversation off the run into
//! its log and keeps whether a loop runs; [`idle`] is the clock,
//! running only while the agent is not active — no loop, and nothing
//! [`Inflight`] — and [`idle_tool`] every tool's, running only
//! while nothing holds the tool, which states every hold there is;
//! [`message`] is the one way into an agent;
//! [`use_tool`] is "one container per tool", deployed for the first
//! thing that uses it and undeployed by its clock `idle_seconds`
//! after the last hold left it — [`connect`] and [`disconnect`] are a
//! connect scope's hold; [`use_connected`] is the same for a connected
//! tool, whose run is this daemon's connect on another daemon's tool,
//! opened by [`connected`]; [`use_dependency`] is the same for a
//! dependency tool, of either mode and scope; [`Holds`] are what
//! holds a tool from outside its run, and [`hold_tools`] and
//! [`release_tools`] an active agent's hold on every tool it is
//! served; [`attached`], [`attached_connected`] and
//! [`detached`] keep running agents' served sets in step with the
//! attachments;
//! [`mcp`] is the one tool list an agent sees and the routing of its
//! calls; [`deploy`] records a container's declared dependencies as
//! dependency tools under their owners and deploys them then and there
//! — a lazy tool of the agent's own, and an eager tool, which
//! [`eager`] keeps for as long as any agent is attached to it, as it
//! keeps a tool on record whose image's mode is eager;
//! [`fuse`] serves the mounts — another provider's volume bridged, or
//! an agent's path into its dependency; [`images`] answers a
//! provider's image asks from the other providers connected. [`Key`]
//! names a container by
//! its record or, a dependency tool, by its row, [`ToolKey`] a
//! tool's run — a standalone tool's, a connected tool's or a
//! dependency tool's; [`AgentRun`] and [`ToolRun`] are what is
//! live for one; [`container`] and [`dependency`] are the request a
//! run is, and
//! [`pump_tool`] reads a tool container's activity to its end;
//! [`events`] writes the graph of who sent what to whom, at the
//! seams the exchanges pass through;
//! [`Opened`] is a run as a file operation works on it, and
//! [`files`] the operations — the container's tree whole, a file
//! read, a file written — with the daemon's mounts included.
//!
//! Its own files are flattened into it, so everything is named
//! through this module and not through the file it lives in.

mod activity;
mod attached;
mod build;
mod error;
pub mod capable;
mod holds;
mod idle;
mod inflight;
mod key;
mod lost;
mod opened;
mod provider;
mod pump;
mod pump_tool;
mod run;
mod start;
mod stop;
mod tools;

pub mod files;

pub mod answerers;
pub mod deploy;
pub mod events;
pub mod fuse;
pub mod images;
pub mod lifecycle;
pub mod mcp;
pub mod message;
pub mod permissions;
pub mod eager;
pub mod tool_log;

pub use activity::*;
pub use attached::*;
pub use build::*;
pub use error::*;
pub use holds::*;
pub use idle::*;
pub use inflight::*;
pub use key::*;
pub use lost::*;
pub use opened::*;
pub use provider::*;
pub use pump::*;
pub use pump_tool::*;
pub use run::*;
pub use start::*;
pub use stop::*;
pub use tools::*;
