//! Sending an image out of the Store, to a registry.
//!
//! A caller names an image it holds, a repository at a registry the
//! provider lists, the tags to put on it there, and the credential to
//! write with; the provider uploads every platform the image has and
//! answers when the registry holds it. It is the Store's one way out:
//! [`push`](super::push) brings an image in, and this sends one on.
//! Split by who SENDS, as everywhere else: the ask is in [`client`],
//! the answer in [`server`].

pub mod client;
pub mod server;
