//! Performing an API request.
//!
//! Only the opening is this endpoint's own: the handle that writes the
//! body and the stream that reads the answer are the
//! [`frontend`](super::super::super::frontend) request's, since the
//! two carry the same input and the same output.

mod execute;

pub use execute::*;

pub use super::super::super::frontend::client::execute::{ExecuteError, ExecuteHandle, ExecuteStream, ExecuteStreamError};
