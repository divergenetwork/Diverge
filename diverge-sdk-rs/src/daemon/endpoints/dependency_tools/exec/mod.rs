//! Running one command in a dependency tool's container, its output streamed.
//!
//! A client names a dependency tool and the command; the daemon runs it in the
//! container and sends its stream — stdout and stderr as they come, then
//! how it ended, as [`exec`](crate::shared::containers::exec) states it —
//! and finishes; or answers that nothing is the one named, forbidden, or
//! that it failed, and finishes. The client's cancel, or the client
//! gone, kills the command. A dependency tool whose container is not running is deployed for the command, and held up while it runs; a tool that mounts its agent's files brings its agent's container up with it, and such a tool its agent then no longer declares is not found.
//! Split by who SENDS: the request and the cancel are in [`client`], the
//! stream in [`server`].

pub mod client;
pub mod server;
