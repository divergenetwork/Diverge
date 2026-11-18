//! Performing a publish, rather than describing it.
//!
//! [`execute`] opens the scope and hands back an [`ExecuteStream`] of
//! what the provider sends — a blob uploaded, as progress, then the
//! image published. A refusal and a failure end the stream as its
//! error.
//!
//! Its own files are flattened into it, so everything is named
//! through this module and not through the file it lives in.

mod error;
mod execute;
mod execute_stream;

pub use error::*;
pub use execute::*;
pub use execute_stream::*;
