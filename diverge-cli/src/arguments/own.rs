//! What `diverge cli` takes.

use clap::Subcommand;

use super::Action;

/// The CLI's own document in the root, `cli.yaml`: worked on whole,
/// or its daemons and which of them is the default read or written by
/// themselves. The CLI holds no lock on the root, and reads the
/// document at every command.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Own {
    /// Read, rewrite or edit the CLI's own document in the root,
    /// `cli.yaml`.
    #[command(subcommand)]
    Config(Action),
    /// List the daemons the CLI may dial, add one, or delete one,
    /// `daemons` in `cli.yaml`.
    #[command(subcommand)]
    Daemons(Daemons),
    /// Read or write which daemon is dialled when a command names
    /// none, `default` in `cli.yaml`.
    #[command(subcommand)]
    Default(DefaultDaemon),
}

/// What is done to the CLI's `daemons`: see
/// [`own`](crate::cli::own). The document keeps them by name; here
/// they are a list, in the document's order, and the name is a member
/// of each.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Daemons {
    /// Print the daemons, one line each, in the document's order:
    /// `{"index":0,"name":"local","address":"ws://127.0.0.1:14980/"}`,
    /// with `identity` where the daemon's own is stated. The index is
    /// what `delete` and `default set` take.
    Get,
    /// Add a daemon, after the last, with no account. Refused where
    /// the name is a daemon's already.
    Add {
        /// The name the daemon has in the document, and on the
        /// command line as `--daemon`.
        #[arg(long)]
        name: String,
        /// Where it is dialled: a WebSocket URL, or a host and port
        /// dialled as `ws://<address>/`.
        #[arg(long)]
        address: String,
    },
    /// Delete the daemon at an index, and its accounts with it.
    /// Refused where it is the default daemon: set another first.
    /// Every daemon after it moves one index down.
    Delete {
        /// The daemon's index, as `get` prints it: `0` is the first.
        #[arg(value_name = "INDEX")]
        index: usize,
    },
}

/// What is done to the CLI's `default`: see [`own`](crate::cli::own).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum DefaultDaemon {
    /// Print the default daemon, on one line:
    /// `{"index":0,"name":"local"}`, or `null` where the document's
    /// `default` names no daemon it has.
    Get,
    /// Make the daemon at an index the default.
    Set {
        /// The daemon's index, as `daemons get` prints it: `0` is the
        /// first.
        #[arg(value_name = "INDEX")]
        index: usize,
    },
}
