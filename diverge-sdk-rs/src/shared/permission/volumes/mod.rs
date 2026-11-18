//! Grants over volumes.
//!
//! What a dependency tool template may ask over volumes, in the
//! shapes [`permission`](crate::shared::permission) states: [`Make`],
//! the actions that bring one into being; [`Over`], the actions over
//! those that exist, reaching all, those listed, or by tags; and [`Permission`], one grant's
//! worth of either, or of tagging. The daemon's grants over volumes
//! are made of the same [`Make`] and [`Over`].

mod key;
mod listed;
mod make;
mod over;
mod permission;

pub use key::*;
pub use listed::*;
pub use make::*;
pub use over::*;
pub use permission::*;
