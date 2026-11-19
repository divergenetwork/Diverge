//! The provider's log: who connected, whether its handshake was
//! taken, what it asked for, and when it went.
//!
//! One short line for each. See [`Log`] for what is written and how
//! much of it is kept.
//!
//! Its own files are flattened into it, so everything is named
//! through this module and not through the file it lives in.

mod log;
mod stamp;

pub use log::*;
pub use stamp::*;
