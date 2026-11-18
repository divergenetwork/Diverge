//! Image publish request data.
//!
//! The image, where it goes, its tags, and the [`Credential`] it is
//! written with — see [`Frame`].

mod credential;
mod frame;

pub use credential::*;
pub use frame::*;
