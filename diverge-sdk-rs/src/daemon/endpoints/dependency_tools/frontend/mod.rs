//! One HTTP request to a dependency tool's frontend.
//!
//! A client names a dependency tool and the request's head; the daemon sends it
//! to the frontend the tool's program serves beneath `/frontend` (see
//! [`frontend`](crate::shared::containers::frontend)), takes the body on
//! the client's channels, and sends the answer — its head, then its
//! body — and finishes; or answers that nothing is the one named,
//! forbidden, that the tool's image declares no frontend, or that it
//! failed, and finishes. A dependency tool whose container is not running is deployed for the request, and held up while it is answered; a tool that mounts its agent's files brings its agent's container up with it, and such a tool its agent then no longer declares is not found. The request does not make the tool active.
//! Split by who SENDS: the request and the channels are in [`client`],
//! the answer in [`server`].

pub mod client;
pub mod server;
