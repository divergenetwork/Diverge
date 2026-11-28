//! A tool program's API: its own HTTP server, beneath `/api`.
//!
//! A tool container's program is one HTTP server on the container's
//! loopback (see [`port`](crate::container_proxy::inside::port)); a
//! tool's image whose `/.diverge/DIVERGE.md` declares `api` answers
//! requests there beneath `/api`. An agent's image declares none, and
//! an agent container is asked for none. It is the second of a tool
//! program's two doors, beside its
//! [`frontend`](crate::shared::containers::frontend): a frontend is
//! pages for a person, an API is whatever HTTP the program speaks, for
//! a caller that knows it.
//!
//! One exchange is one HTTP request, in the frontend's own forms,
//! which are these: its [`request::Head`], then its body; the answer
//! is its [`output`] — the response's head, then its body.
//!
//! The rules:
//!
//! 1. Only a path beneath `/api` is reached, as
//!    [`request::reachable`] states: the program's other routes — its
//!    frontend among them — never are.
//! 2. The request's body is the [`input`]: any number of `Body`, then
//!    `End`.
//! 3. The output is exactly one `Head`, then any number of `Body`; or a
//!    lone `Error`.
//! 4. An answer with status `101` switches the protocol: after it, the
//!    input's `Body` carries the bytes the caller sends and the
//!    output's `Body` the bytes the program sends, until either side
//!    ends.

pub mod request;

pub use super::frontend::{input, output};
