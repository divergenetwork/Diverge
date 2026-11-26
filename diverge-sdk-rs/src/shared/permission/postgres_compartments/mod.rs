//! Grants over the database's compartments.
//!
//! What a dependency tool template may ask over the
//! [compartments](crate::daemon::endpoints::postgres::compartments),
//! in the shapes [`permission`](crate::shared::permission) states:
//! [`Over`], the actions over compartments, reaching all, those
//! listed, or by tags; and [`Permission`], one grant's worth of that,
//! or of tagging. Nothing here makes a compartment: one is on record
//! when what it is of is made. The daemon's grants are made of the
//! same [`Over`].

mod listed;
mod over;
mod permission;

pub use listed::*;
pub use over::*;
pub use permission::*;
