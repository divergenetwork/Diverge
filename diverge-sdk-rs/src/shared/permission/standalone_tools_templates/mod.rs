//! Grants over tool templates.
//!
//! What a dependency tool template may ask over tool templates, in the
//! shapes [`permission`](crate::shared::permission) states: [`Make`],
//! the actions that bring one into being; [`Over`], the actions over
//! those that exist, reaching all, those listed, or by tags; and [`Permission`], one grant's
//! worth of either, or of tagging. The daemon's grants over tool templates
//! are made of the same [`Make`] and [`Over`].

mod listed;
mod make;
mod over;
mod permission;

pub use listed::*;
pub use make::*;
pub use over::*;
pub use permission::*;
