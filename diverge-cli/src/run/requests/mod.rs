//! `diverge run requests`: the requests over requests, as
//! [`requests`](diverge_sdk::daemon::endpoints::requests) states them.

use clap::Subcommand;

use crate::run::{Emitter, Error, Target};

pub mod create;
pub mod listen;

/// The requests over requests.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Requests {
    /// Make a request of accounts and tools, and wait for the one
    /// answer it gets.
    Create(create::Arguments),
    /// Hear the requests made of this account, and answer them on
    /// standard input.
    Listen(listen::Arguments),
}

/// The request put, by its leaf.
pub async fn run(command: Requests, target: &Target<'_>, emitter: &mut Emitter) -> Result<(), Error> {
    match command {
        Requests::Create(arguments) => create::run(arguments, target, emitter).await,
        Requests::Listen(arguments) => listen::run(arguments, target, emitter).await,
    }
}
