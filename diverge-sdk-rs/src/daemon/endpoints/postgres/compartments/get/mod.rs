//! Getting one compartment. One request, one answer. A client names
//! a compartment; the daemon answers with it as a list would report
//! it, that there is none, forbidden, or that it failed, and the scope
//! finishes.
//!
//! Split by who SENDS, as everywhere else. A client asks — so the
//! question is in [`client`] — and the daemon answers, so the answer is
//! in [`server`]. Neither side holds both halves of the exchange.

pub mod client;
pub mod server;
