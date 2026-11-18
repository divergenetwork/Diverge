//! The client side of an image merge: what a client sends.
//!
//! [`request`] is the whole of it. A client asks and then listens; it
//! has nothing to send back.
//!
//! # And a way to use it
//!
//! [`execute`] performs the exchange rather than describing it:
//! hand it a [`Handle`](crate::wire::client::handle::Handle) and a
//! [`request::Frame`], get the image merged.

pub mod request;

pub mod execute;
