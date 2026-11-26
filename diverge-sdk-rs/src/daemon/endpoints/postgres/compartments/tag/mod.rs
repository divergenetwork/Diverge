//! Putting tags on a compartment. One request, one answer. A client names
//! a compartment and the tags; the daemon answers that they are
//! on it, that there is no such compartment, forbidden, or that it
//! failed, and the scope finishes.
//!
//! Split by who SENDS, as everywhere else. A client asks — so the
//! question is in [`client`] — and the daemon answers, so the answer is
//! in [`server`]. Neither side holds both halves of the exchange.

pub mod client;
pub mod server;
