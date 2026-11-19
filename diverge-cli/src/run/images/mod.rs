//! `diverge run images …`.
//!
//! The requests over images: named by digest, kept in the daemon's
//! Stores on its providers. One file per request: the tree of files is
//! the tree of subcommands.

use clap::Subcommand;

use crate::run::{Emitter, Error, Target};

pub mod build;
pub mod delete;
pub mod edit;
pub mod get;
pub mod list;
pub mod merge;
pub mod publish;
pub mod push;
pub mod read;

/// The requests over images.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Images {
    /// Build an image on a provider, from a Containerfile.
    Build(build::Arguments),
    /// Delete an image in a Store of the daemon's, or some of its
    /// platforms.
    Delete(delete::Arguments),
    /// Change the description of an image in a Store of the daemon's.
    Edit(edit::Arguments),
    /// Get one image in a Store of the daemon's.
    Get(get::Arguments),
    /// List the images in the daemon's Stores, and keep the list.
    List(list::Arguments),
    /// Make one image of one or more, in the daemon's Store on a
    /// provider: what the provider lacks is brought onto it from
    /// wherever it is, and the images are merged there.
    Merge(merge::Arguments),
    /// Publish one or more images to a registry as one image: what
    /// the provider lacks is brought onto it, the images are merged
    /// where there are several, and the result is sent.
    Publish(publish::Arguments),
    /// Push an image into the daemon's Store on a provider, from a
    /// registry that provider uses.
    Push(push::Arguments),
    /// Whether the daemon can supply an image, through a provider or a
    /// remote daemon its grants reach, and, where it can, one ask of
    /// it: its DIVERGE.md, one of its icons, a manifest, a blob, or the
    /// proof.
    Read(read::Arguments),
}

/// The request put, by its subcommand.
pub async fn run(command: Images, target: &Target<'_>, emitter: &mut Emitter) -> Result<(), Error> {
    match command {
        Images::Build(arguments) => build::run(arguments, target, emitter).await,
        Images::Delete(arguments) => delete::run(arguments, target, emitter).await,
        Images::Edit(arguments) => edit::run(arguments, target, emitter).await,
        Images::Get(arguments) => get::run(arguments, target, emitter).await,
        Images::List(arguments) => list::run(arguments, target, emitter).await,
        Images::Merge(arguments) => merge::run(arguments, target, emitter).await,
        Images::Publish(arguments) => publish::run(arguments, target, emitter).await,
        Images::Push(arguments) => push::run(arguments, target, emitter).await,
        Images::Read(arguments) => read::run(arguments, target, emitter).await,
    }
}
