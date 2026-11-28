//! Grants over connected tools.
//!
//! What a dependency tool template may ask over connected tools —
//! other daemons' standalone tools, held here — in the shapes
//! [`permission`](crate::shared::permission) states: [`Make`], the
//! action that brings one into being; [`Over`], the actions over those
//! that exist, reaching all, the agent's attached ones, those listed, or by tags; and [`Permission`], one grant's worth
//! of either, or of tagging. The daemon's grants over connected tools
//! are made of the same [`Make`] and [`Over`].

mod key;
mod listed;
mod make;
mod over;
mod permission;
mod within;

pub use key::*;
pub use listed::*;
pub use make::*;
pub use over::*;
pub use permission::*;
pub use within::*;
