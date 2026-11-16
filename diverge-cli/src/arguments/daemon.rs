//! What `diverge daemon` takes.

use clap::Subcommand;

use super::Lifecycle;

/// The daemon on this host: started, stopped, or one setting of its
/// document in the root, `daemon.yaml` — its port, its idle seconds,
/// its administrators, its identity, its Postgres — read or written
/// by itself. The document is not worked on whole: there is no
/// `daemon config`.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Daemon {
    /// Start or stop the daemon: see [`lifecycle`](crate::lifecycle).
    #[command(flatten)]
    Lifecycle(Lifecycle),
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
    /// Read or write who the daemon is, `identity` in `daemon.yaml`.
    #[command(subcommand)]
    Identity(Identity),
    /// Read or write which database the daemon runs on, `postgres`
    /// in `daemon.yaml`.
    #[command(subcommand)]
    Postgres(Postgres),
}

/// What is done to the daemon's `postgres`: see
/// [`setting`](crate::cli::setting).
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Postgres {
    /// Print which database, on one line:
    /// `{"kind":"local","max_connections":1024}`, or
    /// `{"kind":"remote","url":"…","has_password":true}`. The
    /// password is never printed.
    Get,
    /// Write which database, whole: the kind, and what that kind
    /// takes. Refused while the daemon is running on the root: stop
    /// the daemon first.
    #[command(subcommand)]
    Set(PostgresKind),
}

/// The kind of database a `postgres set` writes: each kind is its own
/// command, with that kind's settings as its flags, so that a setting
/// of one kind cannot be given to the other.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum PostgresKind {
    /// A Postgres the daemon runs itself, under the root.
    Local {
        /// The most connections the cluster accepts at once. Absent,
        /// the default, 1024 — not what the document had.
        #[arg(long, value_name = "COUNT")]
        max_connections: Option<u32>,
    },
    /// A Postgres the daemon dials.
    Remote {
        /// Where to dial and as whom, as a Postgres URL with no
        /// password: `postgres://user@host:port/database`.
        #[arg(long)]
        url: String,
        /// The password. Absent, the daemon dials with none — not
        /// with the one the document had.
        #[arg(long)]
        password: Option<String>,
    },
}

/// What is done to a program's `identity`, the daemon's or the
/// provider's: see [`setting`](crate::cli::setting).
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Identity {
    /// Print the identity, on one line: `{"issuer":"…","subject":"…"}`,
    /// or `null` where the document has none.
    Get,
    /// Write the identity, whole: both its issuer and its subject.
    /// Refused while the program is running on the root: stop it
    /// first. There is no setting it to none.
    Set {
        /// The identity's issuer, exactly as its discovery document
        /// names it.
        #[arg(long)]
        issuer: String,
        /// The identity's subject: the `sub` of its tokens.
        #[arg(long)]
        subject: String,
    },
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

/// What is done to a program's `port`, the daemon's or the
/// provider's: see [`setting`](crate::cli::setting).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum Port {
    /// Print the port, as the program reads it: the default where
    /// the document has none.
    Get,
    /// Write the port. Refused while the program is running on the
    /// root: stop it first.
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
