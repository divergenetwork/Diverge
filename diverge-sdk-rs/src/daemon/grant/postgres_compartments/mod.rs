//! Grants over the database's compartments.
//!
//! What a role may allow over the
//! [compartments](crate::daemon::endpoints::postgres::compartments),
//! in the shapes [`grant`](crate::daemon::grant) states: [`Over`], the
//! actions over compartments — defined once in
//! [`shared::permission`](crate::shared::permission) and re-exported —
//! and [`Permission`], one grant's worth of that, or of tagging.

mod permission;

pub use crate::shared::permission::postgres_compartments::Over;
pub use permission::*;
