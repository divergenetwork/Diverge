//! The `containers` section of `provider.yaml`.
//!
//! Whether the provider runs containers, and how much of the host the
//! running set may reach together: [`Containers`]. The section absent,
//! the provider runs none — see [`Containers`].
//!
//! Its own files are flattened into it, so everything is named
//! through this module and not through the file it lives in.

mod containers;

pub use containers::*;
