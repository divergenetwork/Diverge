//! The merge response: [`Frame`], what happened and how it ended;
//! [`Merged`], the image made; and a push's `Blob` and `Added`, which
//! a merge tells as a push does.

mod frame;

pub use frame::*;

pub use crate::daemon::endpoints::images::publish::server::response::Merged;
pub use crate::daemon::endpoints::images::push::server::response::{Added, Blob};
