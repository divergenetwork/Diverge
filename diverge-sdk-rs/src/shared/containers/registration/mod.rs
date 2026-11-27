//! Registering a container with its caller: what its program declared
//! at registration, asked of the caller.
//!
//! A container's program answers its registration with two things:
//! the tools it depends on — each a
//! [`dependencies::Template`](super::dependencies::Template), an
//! agent's alone — and the permissions it holds over the daemon, each
//! a [`Grant`](crate::shared::permission::Grant), a tool's alone. The
//! proxy carries them on the begin's `Begun`;
//! the provider, when either is non-empty, opens one channel on the
//! run scope with [`request::Request`] — both verbatim, with the run's
//! id — and the caller answers with one [`response::Frame`]: deployed,
//! or an error in its own words;
//! then the channel finishes, and only then does the provider send the
//! id. The run ends, the container stopped,
//! on anything but a deploy. A program that declares neither is never
//! asked about, and its id follows at once.
//!
//! What the caller does with the ask is its own: it deploys each
//! dependency, holds the program to its permissions,
//! and answers. The provider carries and does not read any of it.
//! Later activations of the same container take no new registration.

pub mod request;
pub mod response;
