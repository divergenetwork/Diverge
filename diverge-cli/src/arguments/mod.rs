//! The command line: what `diverge` is told.
//!
//! [`Arguments`] is the whole of it — the root and the [`Command`] —
//! parsed by clap, whose help and version are the program's own. The
//! `run` command's own tree — and `request-schema` and
//! `response-schema` beneath each of
//! its requests — is [`run`](crate::run)'s. The provider's, the
//! viewer's and the CLI's document is each worked on beneath its
//! program, `diverge <program> config`, by an
//! [`Action`]: the provider is a [`Server`] — started or stopped by a
//! [`Lifecycle`], or configured — the daemon a [`Daemon`], which is
//! started or stopped likewise, and configured not whole but by its
//! `port`, `idle_seconds`, `admins`, `identity` and `postgres`, each
//! read or written by itself, the viewer
//! and the CLI each a [`Document`], configured alone. The host's
//! OpenID Connect identities are [`Identities`], and its accounts at
//! Diverge's own provider are [`IdentityDivergeNetwork`].
//!
//! Its own files are flattened into it, so everything is named
//! through this module and not through the file it lives in.

mod arguments;
mod command;
mod config;
mod daemon;
mod document;
mod identities;
mod identity_diverge_network;
mod lifecycle;
mod server;

pub use arguments::*;
pub use command::*;
pub use config::*;
pub use daemon::*;
pub use document::*;
pub use identities::*;
pub use identity_diverge_network::*;
pub use lifecycle::*;
pub use server::*;
