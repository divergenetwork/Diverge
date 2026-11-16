//! What `diverge daemon` takes.

use clap::Subcommand;

use super::{Action, Lifecycle};

/// The daemon on this host: started, stopped, its document in the
/// root worked on whole, or one setting of that document — its
/// port, its idle seconds, its administrators — read or written by
/// itself.
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
    /// List the daemon's administrators, add one, or delete one,
    /// `admins` in `daemon.yaml`.
    #[command(subcommand)]
    Admins(Admins),
}

/// What is done to the daemon's `admins`: see
/// [`setting`](crate::cli::setting).
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Admins {
    /// Print the administrators, one line each, in the document's
    /// order: `{"index":0,"issuer":"…","subject":"…"}`. The index is
    /// what `delete` takes.
    Get,
    /// Add an administrator, after the last. Refused while the daemon
    /// is running on the root: stop the daemon first. Refused where
    /// the identity is an administrator already.
    Add {
        /// The identity's issuer, exactly as its discovery document
        /// names it.
        #[arg(long)]
        issuer: String,
        /// The identity's subject: the `sub` of its tokens.
        #[arg(long)]
        subject: String,
    },
    /// Delete the administrator at an index. Refused while the daemon
    /// is running on the root: stop the daemon first. Every
    /// administrator after it moves one index down.
    Delete {
        /// The administrator's index, as `get` prints it: `0` is the
        /// first.
        #[arg(value_name = "INDEX")]
        index: usize,
    },
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
