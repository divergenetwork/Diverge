//! What a provider sends back on an image merge.
//!
//! One frame, once: the image merged, the word that the caller owns
//! it already, a refusal, or a failure — see [`Frame`]. [`Merged`] is
//! what the first carries.

mod frame;
mod merged;

pub use frame::*;
pub use merged::*;
