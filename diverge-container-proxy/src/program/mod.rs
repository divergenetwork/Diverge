//! The program's own server, whichever kind of container: dialled
//! for the calls both kinds answer.
//!
//! Every container's entrypoint is an HTTP server on the loopback, at
//! the port [`diverge_sdk::container_proxy::inside::port()`] names — an
//! agent's loop, or a tool's MCP server — and beside either the call
//! this module makes: `/register` once, when the server begins, with
//! the arguments the container was made with, answered with the
//! dependencies the program declares and the permissions it holds.
//! [`Upstream`] is the client it is made with, and the
//! loop's own calls beside it. What the arguments may be is asked
//! of no program: it is the image's `/.diverge/DIVERGE.md`, which
//! [`info`](crate::info) reads from the container.

mod register;
mod upstream;

pub use register::*;
pub use upstream::*;
