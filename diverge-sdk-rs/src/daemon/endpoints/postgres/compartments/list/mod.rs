//! Listing compartments. A client asks for the compartments, narrowed,
//! and the daemon sends every one its `list` grants reach that the
//! request's filter lets through, one response each, oldest first,
//! then the word that the list is whole, and keeps the scope open:
//! each compartment come to be, changed or gone, as records are made
//! and deleted, as tags change and as connections open and close,
//! until the client cancels, the one channel it opens on the scope. A
//! count keeps the list to the first that many that match. A request
//! that says nothing is every compartment the grants reach.
//!
//! Split by who SENDS, as everywhere else. A client asks — so the
//! question is in [`client`] — and the daemon answers, so the answer is
//! in [`server`]. Neither side holds both halves of the exchange.

pub mod client;
pub mod server;
