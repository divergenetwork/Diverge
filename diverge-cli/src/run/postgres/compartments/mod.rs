//! `diverge run postgres compartments …`.
//!
//! The requests over the database's compartments. One file per request,
//! one folder per family within: the tree of files is the tree of
//! subcommands.

use clap::Subcommand;

use crate::run::{Emitter, Error, Target};

pub mod delete;
pub mod get;
pub mod list;
pub mod tag;
pub mod untag;

/// The requests over the database's compartments.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Compartments {
    /// Deleting a compartment, with every table in it.
    Delete(delete::Arguments),
    /// Getting one compartment.
    Get(get::Arguments),
    /// Listing the compartments, and keeping the list.
    List(list::Arguments),
    /// Putting tags on a compartment.
    Tag(tag::Arguments),
    /// Taking tags off a compartment.
    Untag(untag::Arguments),
}

/// The request put, by its subcommand.
pub async fn run(
    command: Compartments,
    target: &Target<'_>,
    emitter: &mut Emitter,
) -> Result<(), Error> {
    match command {
        Compartments::Delete(arguments) => delete::run(arguments, target, emitter).await,
        Compartments::Get(arguments) => get::run(arguments, target, emitter).await,
        Compartments::List(arguments) => list::run(arguments, target, emitter).await,
        Compartments::Tag(arguments) => tag::run(arguments, target, emitter).await,
        Compartments::Untag(arguments) => untag::run(arguments, target, emitter).await,
    }
}
