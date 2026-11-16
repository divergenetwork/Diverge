//! What `diverge provider` takes.

use clap::Subcommand;

use super::{Action, Lifecycle};

/// The provider on this host: started, stopped, or its document in
/// the root worked on.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Server {
    /// Start or stop the server: see [`lifecycle`](crate::lifecycle).
    #[command(flatten)]
    Lifecycle(Lifecycle),
    /// Read, rewrite or edit the provider's document in the root,
    /// `provider.yaml`.
    #[command(subcommand)]
    Config(Action),
}
