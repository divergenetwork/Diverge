//! What `diverge` does.

use clap::Subcommand;

use super::{Daemon, Identities, IdentityDivergeNetwork, Own, Server, Viewer};
use crate::run::Run;

/// The commands. Every request of the daemon's wire goes under
/// [`Run`](Self::Run). Each program is its own command, and its
/// settings are worked on one at a time, each
/// setting a command beneath [`Daemon`](Self::Daemon),
/// [`Provider`](Self::Provider), [`Viewer`](Self::Viewer) or
/// [`Cli`](Self::Cli); none is worked on whole. The daemon and the provider on this host
/// are also started and stopped under their commands. The host's identities are
/// [`Identities`](Self::Identities).
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// Put one request to a daemon, as an account of it: see
    /// [`run`](crate::run).
    Run(Run),
    /// Sign in, list or remove an OpenID Connect identity under the
    /// root, the one store of identities every Diverge program on the
    /// host shares. No command prints a token.
    #[command(subcommand)]
    Identities(Identities),
    /// Work on this host's accounts at `identity.diverge.network`,
    /// Diverge's own OpenID Connect provider, and ask it as one: see
    /// [`identity_diverge_network`](crate::cli::identity_diverge_network).
    #[command(name = "identity.diverge.network", subcommand)]
    IdentityDivergeNetwork(IdentityDivergeNetwork),
    /// Start or stop the daemon on the root, or read or write one
    /// setting of it: see
    /// [`lifecycle`](crate::lifecycle) and
    /// [`setting`](crate::cli::setting).
    #[command(subcommand)]
    Daemon(Daemon),
    /// Start or stop the provider on the root, or read or write one
    /// setting of it: see
    /// [`lifecycle`](crate::lifecycle) and
    /// [`provider`](crate::cli::provider).
    #[command(subcommand)]
    Provider(Server),
    /// Start or stop the viewer on the root, or read or write one
    /// setting of it: see [`lifecycle`](crate::lifecycle),
    /// [`viewer`](crate::cli::viewer) and [`own`](crate::cli::own).
    #[command(subcommand)]
    Viewer(Viewer),
    /// Read or write one setting of the CLI itself — its
    /// daemons, what each is, and which is the default: see
    /// [`own`](crate::cli::own).
    #[command(subcommand)]
    Cli(Own),
}
