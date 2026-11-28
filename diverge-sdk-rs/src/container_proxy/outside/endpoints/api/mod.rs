//! One HTTP request to the program's API, beneath `/api`.
//!
//! The server opens it with the request's head and gets the program's
//! response — its head, then its body — until it ends or the server
//! cancels it; it sends the request's body on channels. The request, its
//! answer and its rules are [`api`](crate::shared::containers::api)'s,
//! which are a [`frontend`](super::frontend) request's beneath another
//! prefix, so everything here but the tag that opens the scope is that
//! endpoint's own.

pub mod client;
pub mod server;
