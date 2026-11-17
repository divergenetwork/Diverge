//! The viewer's settings, as the CLI reads and writes them.

use diverge_sdk::wire::proxy::Redial;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::{Daemon, LOCAL};

/// The loopback port a viewer hears its stop on unless its settings
/// name another: one above the port a daemon listens on.
pub const VIEWER_PORT: u16 = diverge_sdk::daemon::DEFAULT_PORT + 1;

/// The file under the root a running viewer holds locked for its
/// life, with its process id inside.
pub const VIEWER_LOCK_FILE: &str = "viewer.lock";

/// Everything the viewer is told, member for member what the app's
/// own type states (`diverge-viewer/src-tauri/src/config/`): the CLI
/// cannot take a type from a Tauri app, so the two state one
/// document, and a change to either's fields is a change to both.
///
/// Its `daemons` and `default` are the CLI's own, with the CLI's own
/// [`Daemon`]: the viewer dials a daemon as the CLI does, by a name,
/// as an account of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Viewer {
    /// The port, on the loopback only, the app hears its stop on.
    pub port: u16,
    /// Whether, and how long after, the app dials the daemon again:
    /// `never`, or a whole number of seconds.
    #[serde(skip_serializing_if = "Redial::is_default")]
    pub redial: Redial,
    /// The daemon dialled when a start names none, by its name in
    /// `daemons`.
    pub default: String,
    /// The daemons the viewer may dial, by name, in the order they
    /// were added.
    pub daemons: IndexMap<String, Daemon>,
}

/// The built-in default: the stop heard on [`VIEWER_PORT`], a redial
/// of five seconds, and one daemon, [`LOCAL`] — the daemon on this
/// host at a daemon's own default port, the CLI's default daemon too
/// — with no account.
impl Default for Viewer {
    fn default() -> Self {
        let local = Daemon {
            address: format!("ws://127.0.0.1:{}/", diverge_sdk::daemon::DEFAULT_PORT),
            default: None,
            identity: None,
            accounts: IndexMap::new(),
        };
        Viewer {
            port: VIEWER_PORT,
            redial: Redial::default(),
            default: LOCAL.to_string(),
            daemons: IndexMap::from([(LOCAL.to_string(), local)]),
        }
    }
}
