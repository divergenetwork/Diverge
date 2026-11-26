//! Deleting a compartment. One request, one answer. A client names a
//! compartment; the daemon answers that it is gone, with every table
//! in it, that there is none, that it is not one a delete removes
//! now, that a container holds it, forbidden, or that it failed, and
//! the scope finishes. Which compartments a delete removes, and when,
//! is stated in [`compartments`](super).
//!
//! Split by who SENDS, as everywhere else. A client asks — so the
//! question is in [`client`] — and the daemon answers, so the answer is
//! in [`server`]. Neither side holds both halves of the exchange.

pub mod client;
pub mod server;
