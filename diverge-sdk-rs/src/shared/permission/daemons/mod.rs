//! Grants over remote daemons.
//!
//! The remote daemons are the daemon's records of them, each one account
//! of ours on another daemon. What a dependency tool template may ask
//! over them, in the shapes [`permission`](crate::shared::permission)
//! states: [`Make`], the action that records one; [`Over`], the actions
//! over the records — reading, changing, deleting, and holding a
//! connected tool on one or relaying to one — reaching all, those listed, or by tags; and
//! [`Permission`], one grant's worth of either, or of tagging.

mod listed;
mod make;
mod over;
mod permission;

pub use listed::*;
pub use make::*;
pub use over::*;
pub use permission::*;
