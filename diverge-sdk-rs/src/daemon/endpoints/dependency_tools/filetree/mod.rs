//! Watching a dependency tool's container's filesystem, whole.
//!
//! A client names a dependency tool — by its template and whose it is
//! — and the daemon sends a snapshot of the container's whole tree —
//! every mount in it, the FUSE mounts the daemon itself serves spliced
//! in at their mount points — and then one frame per change, for as
//! long as the scope lives, and finishes when the client cancels or the
//! tool's run ends; or answers that no dependency tool is the one named, forbidden, or that it failed, and finishes. The tree
//! is the provider protocol's [`filetree`](crate::shared::filetree)
//! stream, which leaves the FUSE mounts out, made whole by the daemon:
//! each mount's subtree from the filetree channel of the
//! `volumes::fuse_serve` the daemon bridges the mount to, and a change under
//! one from the same source. A dependency tool whose container is
//! not running is deployed for the watch, and held up for as long as
//! the watch is open; a tool that mounts its agent's files brings its
//! agent's container up with it, and such a tool its agent then no longer declares
//! is `NotFound`.
//!
//! Split by who SENDS, as everywhere else. A client asks — so the
//! question and the cancel are in [`client`] — and the daemon answers,
//! so the answer is in [`server`]. Neither side holds both halves of
//! the exchange.

pub mod client;
pub mod server;
