//! What `diverge` does.

use clap::Subcommand;

use super::{Daemon, Identities, IdentityDivergeNetwork, Own, Server, Viewer};
use crate::run::Run;

/// The commands. Every request of the daemon's wire goes under
/// [`Run`](Self::Run). Each program is its own command. The CLI's
/// own document in the root is worked on whole, as `config`, beneath
/// [`Cli`](Self::Cli); the daemon's, the provider's and the viewer's
/// are worked on one setting at a time, each setting a command
/// beneath [`Daemon`](Self::Daemon), [`Provider`](Self::Provider) or
/// [`Viewer`](Self::Viewer). The daemon and the provider on this host
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
    /// setting of its `daemon.yaml`: see
    /// [`lifecycle`](crate::lifecycle) and
    /// [`setting`](crate::cli::setting).
    #[command(subcommand)]
    Daemon(Daemon),
    /// Start or stop the provider on the root, or work on its
    /// `provider.yaml`: see [`lifecycle`](crate::lifecycle) and
    /// [`config`](crate::cli::config).
    #[command(subcommand)]
    Provider(Server),
    /// Read or write one setting of the viewer's `viewer.yaml`: see
    /// [`viewer`](crate::cli::viewer).
    #[command(subcommand)]
    Viewer(Viewer),
    /// Work on the CLI's own `cli.yaml`, whole or its daemons and its
    /// default daemon by themselves: see
    /// [`config`](crate::cli::config) and [`own`](crate::cli::own).
    #[command(subcommand)]
    Cli(Own),
}
