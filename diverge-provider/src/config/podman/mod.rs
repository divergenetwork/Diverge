//! The `podman` section of `provider.yaml`.
//!
//! The podman the provider uses, for whatever it uses it for:
//! [`Podman`] — the registries the provider looks in for an image,
//! where podman keeps its data, and how much the image cache may
//! hold. It is read only by a provider that uses podman at all, which
//! is one with a [`containers`](super::containers) section or an
//! [`images`](super::images) section; a provider with neither runs no
//! podman, and the section says nothing to it. The file may leave the
//! section out, or any field of it, and each is then its default.
//!
//! Its own files are flattened into it, so everything is named
//! through this module and not through the file it lives in.

mod credential;
mod podman;
mod registry;

pub use credential::*;
pub use podman::*;
pub use registry::*;
