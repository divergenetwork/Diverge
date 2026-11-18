//! Publish request data. What a caller hands the daemon to publish:
//! the parts, where they go, the tags, the credential, and what the
//! image made is kept as.

mod credential;
mod frame;
mod part;

pub use credential::*;
pub use frame::*;
pub use part::*;
