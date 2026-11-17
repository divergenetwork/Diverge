//! The document itself.

use serde::{Deserialize, Serialize};

use super::Daemon;

/// The whole of `viewer.yaml`, as Diverge Viewer reads it: what the app
/// is told.
///
/// An absent file is this type's [`Default`]. An unknown key is an
/// error, so a misspelled setting is refused rather than ignored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    /// The daemon the app dials, and as whom: see [`Daemon`]. Read at
    /// start; another daemon is another start. Always there: a
    /// document that leaves it out, or leaves a member of it out, has
    /// that member's default — see [`Daemon`]'s. The defaults name no
    /// identity, and the app does not start until one is named.
    pub daemon: Daemon,
}

/// A document with no line in it: the daemon on this host, dialled
/// as nobody, so the app refuses to start until an identity is named.
impl Default for Config {
    fn default() -> Self {
        Config { daemon: Daemon::default() }
    }
}
