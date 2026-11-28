//! An API request, its body this command's stdin and its answer this
//! command's output.
//!
//! An API request is a frontend request beneath another prefix: the
//! same body in, the same head and body out, the same refusals but
//! for the one that says the image declares no such door. So its
//! shape is the [`frontend`](super::frontend)'s own, under this name:
//! the request's body is this command's stdin, the response's head is
//! one JSON line on stderr, its body goes to stdout raw, and the exit
//! code is `0` for a response that ended whole and `1` otherwise.

pub use super::frontend::{Said, frontend as api};
