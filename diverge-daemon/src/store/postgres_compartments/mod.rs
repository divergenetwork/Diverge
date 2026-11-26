//! The compartment records: every compartment of the database a
//! container reaches, on record from the moment what it is of is made.
//!
//! [`Entry`] is one as the store holds it: its role, which names it,
//! the compartment whole, what kind of container it is of, its tags
//! and since when. [`all`] and [`by_role`] load; [`register`] puts one
//! on record, or leaves the one there is; [`set_tags`] writes the
//! tags; [`delete`] removes the record, the role and the schema being
//! the caller's to drop beside it. [`Holds`] is what keeps the
//! compartments a delete of their own removes from being deleted,
//! loaded once for many.
//!
//! Its own files are flattened into it, so everything is named
//! through this module and not through the file it lives in.

mod entry;
mod holds;
mod load;
mod write;

pub use entry::*;
pub use holds::*;
pub use load::*;
pub use write::*;
