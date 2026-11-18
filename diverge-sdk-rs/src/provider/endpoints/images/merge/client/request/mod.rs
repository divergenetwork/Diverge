//! Image merge request data.
//!
//! The sources, each a [`Part`], and what the caller says of the
//! result — see [`Frame`].

mod frame;
mod part;

pub use frame::*;
pub use part::*;
