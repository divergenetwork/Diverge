//! What `diverge cli` takes.

use clap::{Args, Subcommand};

use super::{Identity, Text};

/// The CLI's own document in the root, `cli.yaml`: its daemons, what
/// each of them is, and which of them is the default, each read or
/// written by itself. The document is not worked on whole: there is
/// no `cli config`. The CLI holds no lock on the root, and reads the
/// document at every command.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Own {
    /// List the daemons the CLI may dial, add one, delete one, or —
    /// after a daemon's index — read or write one setting of that
    /// daemon, `daemons` in `cli.yaml`.
    Daemons(Daemons),
    /// Read or write which daemon is dialled when a command names
    /// none, `default` in `cli.yaml`.
    #[command(subcommand)]
    Default(Chosen),
}

/// `diverge cli daemons …`: the list, or one daemon of it. With no
/// index, the list is worked on — `get`, `add`, `delete`. With a
/// daemon's index first, one setting of that daemon is —
/// `diverge cli daemons 0 address get`.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct Daemons {
    /// The index of the daemon whose setting is read or written, as
    /// `daemons get` prints it: `0` is the first. Left out for `get`,
    /// `add` and `delete`, which work on the list.
    #[arg(value_name = "INDEX")]
    pub index: Option<usize>,
    /// What to do.
    #[command(subcommand)]
    pub command: DaemonsCommand,
}

/// What is done to the CLI's `daemons`, or to one of them: see
/// [`own`](crate::cli::own). The document keeps the daemons by name;
/// here they are a list, in the document's order, and the name is a
/// member of each.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum DaemonsCommand {
    /// The list: print the daemons, one line each, in the document's
    /// order: `{"index":0,"name":"local"}`. The index is what a
    /// setting of one, `delete` and `default set` take. What a daemon
    /// is, is read a setting at a time.
    Get,
    /// The list: add a daemon, after the last, with no account.
    /// Refused where the name is a daemon's already.
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
    /// The list: delete the daemon at an index, and its accounts with
    /// it. Refused where it is the default daemon: set another first.
    /// Every daemon after it moves one index down.
    Delete {
        /// The daemon's index, as `get` prints it: `0` is the first.
        #[arg(value_name = "INDEX")]
        index: usize,
    },
    /// One daemon: read or write where it is dialled, its `address`.
    #[command(subcommand)]
    Address(Text),
    /// One daemon: read or write which of its accounts is dialled as
    /// when a command names none, its `default`.
    #[command(subcommand)]
    Default(Chosen),
    /// One daemon: read or write the daemon's own identity, which
    /// must answer at the address where it is stated, its `identity`.
    #[command(subcommand)]
    Identity(Identity),
    /// One daemon: list the accounts the CLI may dial it as, add one,
    /// or delete one, its `accounts`.
    #[command(subcommand)]
    Accounts(DaemonAccounts),
}

/// What is done to a default that is one of a list — the default
/// daemon among the daemons, a daemon's default account among its
/// accounts — said by its index in that list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum Chosen {
    /// Print the default, on one line: `{"index":0,"name":"local"}`,
    /// or `null` where none is named, or the one named is not in the
    /// list.
    Get,
    /// Make the one at an index the default.
    Set {
        /// Its index, as the list's `get` prints it: `0` is the first.
        #[arg(value_name = "INDEX")]
        index: usize,
    },
}

/// What is done to one daemon's `accounts`: see
/// [`own`](crate::cli::own). The document keeps them by name; here
/// they are a list, as the daemons are.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum DaemonAccounts {
    /// Print the accounts, one line each, in the document's order:
    /// `{"index":0,"name":"admin","issuer":"…","subject":"…"}`. The
    /// index is what `delete` and the daemon's `default set` take.
    Get,
    /// Add an account, after the last. Refused where the name is an
    /// account's of the daemon already.
    Add {
        /// The name the account has in the document, and on the
        /// command line as `--account`.
        #[arg(long)]
        name: String,
        /// The identity's issuer, exactly as its discovery document
        /// names it.
        #[arg(long)]
        issuer: String,
        /// The identity's subject: the `sub` of its tokens.
        #[arg(long)]
        subject: String,
    },
    /// Delete the account at an index. Refused where it is the
    /// daemon's default account: set another first. Every account
    /// after it moves one index down.
    Delete {
        /// The account's index, as `get` prints it: `0` is the first.
        #[arg(value_name = "INDEX")]
        index: usize,
    },
}
