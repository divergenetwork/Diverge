//! What `diverge provider` takes.

use clap::Subcommand;

use super::{Action, Identity, Lifecycle, Port};

/// The provider on this host: started, stopped, its document in the
/// root worked on whole, or one setting of that document — its port,
/// its identity, whom it accepts, whom it dials, the registries it
/// pulls from — read or written by itself.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Server {
    /// Start or stop the server: see [`lifecycle`](crate::lifecycle).
    #[command(flatten)]
    Lifecycle(Lifecycle),
    /// Read, rewrite or edit the provider's document in the root,
    /// `provider.yaml`.
    #[command(subcommand)]
    Config(Action),
    /// Read or write the TCP port the provider accepts the protocol
    /// on, `port` in `provider.yaml`.
    #[command(subcommand)]
    Port(Port),
    /// Read or write who the provider is, `identity` in
    /// `provider.yaml`.
    #[command(subcommand)]
    Identity(Identity),
    /// List the identities the provider accepts, add one, or delete
    /// one, `incoming` in `provider.yaml`.
    #[command(subcommand)]
    Incoming(Incoming),
    /// List the peers the provider dials, add one, or delete one,
    /// `outgoing` in `provider.yaml`.
    #[command(subcommand)]
    Outgoing(Outgoing),
    /// Read or write one setting of the podman the provider uses,
    /// `podman` in `provider.yaml`.
    #[command(subcommand)]
    Podman(Podman),
}

/// The settings of the provider's `podman` section, each a command
/// beneath `diverge provider podman`, named as the section names it.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Podman {
    /// List the registries the provider pulls from, add one, or
    /// delete one, `podman.registries` in `provider.yaml`.
    #[command(subcommand)]
    Registries(Registries),
}

/// What is done to the provider's `podman.registries`: see
/// [`provider`](crate::cli::provider).
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Registries {
    /// Print the registries, one line each, in the document's order:
    /// `{"index":0,"host":"docker.io"}`. The index is what `delete`
    /// takes.
    Get,
    /// Add a registry, after the last. Refused while the provider is
    /// running on the root: stop it first. Refused where the host is
    /// listed already.
    Add {
        /// The registry's host, `docker.io`, `ghcr.io`,
        /// `registry.example.com:5000`: what an image reference names
        /// before its first `/`, with no scheme and no path.
        #[arg(value_name = "HOST")]
        host: String,
    },
    /// Delete the registry at an index. Refused while the provider is
    /// running on the root: stop it first. Every registry after it
    /// moves one index down.
    Delete {
        /// The registry's index, as `get` prints it: `0` is the first.
        #[arg(value_name = "INDEX")]
        index: usize,
    },
}

/// What is done to the provider's `incoming`: see
/// [`provider`](crate::cli::provider).
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Incoming {
    /// Print the identities the provider accepts, one line each, in
    /// the document's order: `{"index":0,"issuer":"…","subject":"…"}`.
    /// The index is what `delete` takes.
    Get,
    /// Add an identity, after the last. Refused while the provider is
    /// running on the root: stop it first. Refused where the identity
    /// is listed already.
    Add {
        /// The identity's issuer, exactly as its discovery document
        /// names it.
        #[arg(long)]
        issuer: String,
        /// The identity's subject: the `sub` of its tokens.
        #[arg(long)]
        subject: String,
    },
    /// Delete the identity at an index. Refused while the provider is
    /// running on the root: stop it first. Every identity after it
    /// moves one index down.
    Delete {
        /// The identity's index, as `get` prints it: `0` is the first.
        #[arg(value_name = "INDEX")]
        index: usize,
    },
}

/// What is done to the provider's `outgoing`: see
/// [`provider`](crate::cli::provider).
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Outgoing {
    /// Print the peers the provider dials, one line each, in the
    /// document's order:
    /// `{"index":0,"address":"…","issuer":"…","subject":"…"}`. The
    /// index is what `delete` takes.
    Get,
    /// Add a peer, after the last. Refused while the provider is
    /// running on the root: stop it first. Refused where the same
    /// identity at the same address is listed already.
    Add {
        /// Where the peer listens: `host:port`, dialled as
        /// `ws://<address>/`, or a URL with its own scheme.
        #[arg(long)]
        address: String,
        /// The issuer of the identity that must answer there, exactly
        /// as its discovery document names it.
        #[arg(long)]
        issuer: String,
        /// The subject of the identity that must answer there: the
        /// `sub` of its tokens.
        #[arg(long)]
        subject: String,
    },
    /// Delete the peer at an index. Refused while the provider is
    /// running on the root: stop it first. Every peer after it moves
    /// one index down.
    Delete {
        /// The peer's index, as `get` prints it: `0` is the first.
        #[arg(value_name = "INDEX")]
        index: usize,
    },
}
