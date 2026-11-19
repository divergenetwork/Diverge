//! The `volumes` section of `provider.yaml`.
//!
//! Three kinds of thing: the [`Store`]s new volumes are created in,
//! each with a capacity in bytes; the [`Fixed`] volumes that exist
//! before any client asks, each offered to every identity or to those
//! a hook vouches for; and [`EphemeralFuse`], where a FUSE serve of an
//! ephemeral volume keeps what is written through it. [`Volumes`]
//! holds all three.
//!
//! Its own files are flattened into it, so everything is named
//! through this module and not through the file it lives in.

mod ephemeral_fuse;
mod fixed;
mod store;
mod volumes;

pub use ephemeral_fuse::*;
pub use fixed::*;
pub use store::*;
pub use volumes::*;
