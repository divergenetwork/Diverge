//! The command line: what `diverge` is told.
//!
//! [`Arguments`] is the whole of it — the root and the [`Command`] —
//! parsed by clap, whose help and version are the program's own. The
//! `run` command's own tree — and `request-schema` and
//! `response-schema` beneath each of
//! its requests — is [`run`](crate::run)'s. The CLI's own document
//! is worked on whole, `diverge cli config`, by an [`Action`], and
//! its daemons and its default daemon by themselves: it is an
//! [`Own`]. Every other program's is configured not whole but
//! one setting at a time, each setting a command of its own that
//! reads it or writes it by itself: the daemon is a [`Daemon`] and
//! the provider a [`Server`], each also started or stopped by a
//! [`Lifecycle`], and the viewer a [`Viewer`]. The host's
//! OpenID Connect identities are [`Identities`], and its accounts at
//! Diverge's own provider are [`IdentityDivergeNetwork`].
//!
//! Its own files are flattened into it, so everything is named
//! through this module and not through the file it lives in.

mod arguments;
mod command;
mod config;
mod daemon;
mod identities;
mod identity_diverge_network;
mod lifecycle;
mod own;
mod server;
mod viewer;

pub use arguments::*;
pub use command::*;
pub use config::*;
pub use daemon::*;
pub use identities::*;
pub use identity_diverge_network::*;
pub use lifecycle::*;
pub use own::*;
pub use server::*;
pub use viewer::*;
