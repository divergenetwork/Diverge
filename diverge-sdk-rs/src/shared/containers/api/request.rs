//! What opens an API request: its method, its path and its headers —
//! a frontend request's own forms, beneath another prefix.

pub use crate::shared::containers::frontend::request::{Head, Request};

/// The prefix every reachable path begins with.
pub const PREFIX: &str = "/api";

/// Whether `head`'s path is one an API request may reach: one
/// [`beneath`](Head::beneath) [`PREFIX`], by the rules a frontend
/// request's path is held to beneath its own.
pub fn reachable(head: &Head) -> bool {
    head.beneath(PREFIX)
}
