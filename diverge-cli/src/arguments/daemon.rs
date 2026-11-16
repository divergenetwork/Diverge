//! What `diverge daemon` takes.

use clap::Subcommand;

use super::{Action, Lifecycle};

/// The daemon on this host: started, stopped, its document in the
/// root worked on whole, or one setting of that document read or
/// written by itself.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Daemon {
    /// Start or stop the daemon: see [`lifecycle`](crate::lifecycle).
    #[command(flatten)]
    Lifecycle(Lifecycle),
    /// Read, rewrite or edit the daemon's document in the root,
    /// `daemon.yaml`.
    #[command(subcommand)]
    Config(Action),
    /// Read or write the TCP port the daemon accepts the protocol on,
    /// `port` in `daemon.yaml`.
    #[command(subcommand)]
    Port(Port),
    /// Read or write how many seconds a container lies unused before
    /// the daemon ends its run, `idle_seconds` in `daemon.yaml`.
    #[command(name = "idle_seconds", subcommand)]
    IdleSeconds(IdleSeconds),
}

/// What is done to the daemon's `port`: see
/// [`setting`](crate::cli::setting).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum Port {
    /// Print the port, as the daemon reads it: the default where the
    /// document has none.
    Get,
    /// Write the port. Refused while the daemon is running on the
    /// root: stop the daemon first.
    Set {
        /// The port.
        #[arg(value_name = "PORT")]
        value: u16,
    },
}

/// What is done to the daemon's `idle_seconds`: see
/// [`setting`](crate::cli::setting).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum IdleSeconds {
    /// Print the number of seconds, as the daemon reads it: the
    /// default where the document has none.
    Get,
    /// Write the number of seconds. Refused while the daemon is
    /// running on the root: stop the daemon first.
    Set {
        /// The number of seconds.
        #[arg(value_name = "SECONDS")]
        value: u64,
    },
}
