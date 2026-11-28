//! One HTTP request to a dependency tool's API.
//!
//! A client names a dependency tool and the request's head; the daemon sends it
//! to the API the tool's program serves beneath `/api` (see
//! [`api`](crate::shared::containers::api)), takes the body on
//! the client's channels, and sends the answer — its head, then its
//! body — and finishes; or answers that nothing is the one named,
//! forbidden, that the tool's image declares no API, or that it
//! failed, and finishes. A dependency tool whose container is not running is deployed for the request, and held up while it is answered; a tool that mounts its agent's files brings its agent's container up with it, and such a tool its agent then no longer declares is not found. The request does not make the tool active.
//! Whoever asks — an account, an agent's container or a tool's, each under the same grant — the program is told who, in the request's `diverge.network` header: the [`Caller`](crate::shared::mcp::attestation::Caller) as the daemon attests it, set in place of any the request carried.
//! Split by who SENDS: the request and the channels are in [`client`],
//! the answer in [`server`].

pub mod client;
pub mod server;
