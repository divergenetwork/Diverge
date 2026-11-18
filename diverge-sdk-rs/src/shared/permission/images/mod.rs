//! Over images: the daemon's Stores, by the tags of their providers.
//!
//! An image is no record of the daemon's: a request names one by its
//! digest. What the daemon does hold is a Store on each provider that
//! keeps one, and that is what a grant over images reaches: an object
//! of the [`Action`]s held and how far they reach,
//! `{"images":{"actions":["build","list"],"within":"any"}}`. Here
//! `within` is `"any"`, the providers [`Listed`] by identity, or the
//! shared [`Tags`](crate::shared::permission::Tags) test over the
//! providers' records' tags, as it is for
//! [`providers`](crate::shared::permission::providers) and
//! [`providers_daemons`](crate::shared::permission::providers_daemons);
//! the daemon's own grant may also name providers by identity: see
//! [`daemon::grant::images`](crate::daemon::grant::images), which
//! states how each action is judged.
//!
//! [`Permission`] is that object as a type.

mod action;
mod listed;
mod permission;

pub use action::*;
pub use listed::*;
pub use permission::*;
