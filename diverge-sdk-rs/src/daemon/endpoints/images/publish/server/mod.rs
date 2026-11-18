//! The server side of a publish: what the daemon sends.
//!
//! [`response`] is the whole of it. The daemon opens no channel of
//! its own for this, so there is no `request` here.

pub mod response;
