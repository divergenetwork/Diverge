//! The list response: a compartment added, changed or removed, the
//! word that the list is whole, forbidden, or a failure.
//!
//! [`Frame`] is what a response frame holds; [`Listed`] is what one
//! compartment is.

mod frame;
mod listed;

pub use frame::*;
pub use listed::*;
