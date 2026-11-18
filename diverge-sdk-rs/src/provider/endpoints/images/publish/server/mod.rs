//! The server side of an image publish: what a provider sends.
//!
//! [`response`] is the whole of it. A provider uploads, says what it
//! uploaded as it goes, answers, and is done; it opens no channel of
//! its own.

pub mod response;
