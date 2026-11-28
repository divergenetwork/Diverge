//! One HTTP request to a standalone tool's API.
//!
//! A client names a standalone tool and the request's head; the daemon sends it
//! to the API the tool's program serves beneath `/api` (see
//! [`api`](crate::shared::containers::api)), takes the body on
//! the client's channels, and sends the answer — its head, then its
//! body — and finishes; or answers that nothing is the one named,
//! forbidden, that the tool's image declares no API, or that it
//! failed, and finishes. A tool's container that is not running is started for the request, and held up while it is answered. A connected tool's API is the other daemon's, and no request here finds one.
//! Whoever asks — an account, an agent's container or a tool's, each under the same grant — the program is told who, in the request's `diverge.network` header: the [`Caller`](crate::shared::mcp::attestation::Caller) as the daemon attests it, set in place of any the request carried.
//! Split by who SENDS: the request and the channels are in [`client`],
//! the answer in [`server`].

pub mod client;
pub mod server;
