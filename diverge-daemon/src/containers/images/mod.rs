//! Images answered from the other providers and the remote daemons: a
//! provider's asks on a run scope, and a client's on the daemon's own
//! `images::read` scope — whether an image can be supplied, its
//! manifests, its blobs, the proof, its document — answered by finding
//! the sources that can supply the image, and reading from there,
//! through the `images::read` scopes that answered held.
//!
//! The daemon holds no image itself. [`search`] asks every provider and
//! every remote daemon in reach at once whether it has the image, and
//! hands back each source as it answers with the platforms it adds,
//! cancelling every other once every platform wanted is found;
//! [`locate`] is the first of them. [`manifest`], [`blob`] and
//! [`prove`] pull through the sources kept in [`Pulls`], each part
//! from the source that has it, and answer not held when none does; [`info`] is the image's
//! `/.diverge/DIVERGE.md`, from the daemon's record or read through a source,
//! and nowhere else; [`name`] is the repository path the
//! daemon answers a has with, which the asking provider's own registry
//! serves the image as and never reads.
//!
//! Its own files are flattened into it, so everything is named
//! through this module and not through the file it lives in.

mod icon;
mod info;
mod locate;
mod name;
mod pull;

pub use icon::*;
pub use info::*;
pub use locate::*;
pub use name::*;
pub use pull::*;
