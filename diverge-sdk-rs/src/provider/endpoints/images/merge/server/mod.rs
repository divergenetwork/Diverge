//! The server side of an image merge: what a provider sends.
//!
//! [`response`] is the whole of it. A provider writes the image,
//! answers, and is done; it opens no channel of its own.

pub mod response;
