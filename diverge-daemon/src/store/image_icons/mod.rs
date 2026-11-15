//! The icons the daemon has read of each image, by digest.
//!
//! A digest is the image, so each icon an image ships is fixed by it:
//! an icon read once is that icon for good, and nothing here is ever
//! updated or removed. Only an icon is recorded: that an image ships
//! none, that one is too large, and a failure have no row, and are
//! asked again. [`by_digest`] reads what is on record; [`save`] puts an
//! icon on record.
//!
//! Its own files are flattened into it, so everything is named
//! through this module and not through the file it lives in.

mod load;
mod save;
mod which;

pub use load::*;
pub use save::*;
pub use which::*;
