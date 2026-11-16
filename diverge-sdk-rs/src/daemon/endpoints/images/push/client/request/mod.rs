//! Push request data. What a caller hands the daemon to push an
//! image: the provider, the image, where its bytes come from, and
//! what is said of it.

mod frame;
mod source;

pub use frame::*;
pub use source::*;
