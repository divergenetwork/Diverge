//! The command line: what `diverge` is told.
//!
//! [`Arguments`] is the whole of it — the root and the [`Command`] —
//! parsed by clap, whose help and version are the program's own. The
//! `run` command's own tree — and `request-schema` and
//! `response-schema` beneath each of
//! its requests — is [`run`](crate::run)'s. The viewer's and the
//! CLI's document is each worked on whole beneath its program,
//! `diverge <program> config`, by an [`Action`]: the CLI's is a
//! [`Document`], configured alone, and the viewer's a [`Viewer`],
//! which is that and the settings of its `daemon` each read or
//! written by itself. The daemon is a [`Daemon`] and the
//! provider a [`Server`]: each started or stopped by a [`Lifecycle`],
//! and configured not whole but one setting at a time, each setting a
//! command of its own that reads it or writes it by itself. The host's
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
mod viewer;

pub use arguments::*;
pub use command::*;
pub use config::*;
pub use daemon::*;
pub use document::*;
pub use identities::*;
pub use identity_diverge_network::*;
pub use lifecycle::*;
pub use server::*;
pub use viewer::*;
