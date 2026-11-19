//! Requests pending on the daemon, and who is listening for them.
//!
//! A request is made by a `requests create` scope and lives exactly as
//! long as that scope; a listener is a `requests listen` scope. Both
//! are in memory and nowhere else: see [`Requests`], which is all the
//! state there is, and [`Party`], who a listener is.
//!
//! Its own files are flattened into it, so everything is named
//! through this module and not through the file it lives in.

mod hub;
mod party;

pub use hub::*;
pub use party::*;
