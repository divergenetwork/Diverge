//! The viewer's settings themselves.

use diverge_sdk::wire::proxy::Redial;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::Daemon;

/// The name the built-in default daemon has, as the CLI's has.
pub const LOCAL: &str = "local";

/// The loopback port a viewer hears its stop on unless its settings
/// name another: one above the port a daemon listens on, so that a
/// provider, a daemon and a viewer on one host do not contend.
pub const DEFAULT_PORT: u16 = diverge_sdk::daemon::DEFAULT_PORT + 1;

/// The file under the root a running viewer holds locked for its
/// life, with its process id inside: one viewer per root, and what
/// the CLI probes before it writes a setting, starts or stops one.
pub const LOCK_FILE: &str = "viewer.lock";

/// Everything the app is told: the daemons it may dial, by name, as
/// the CLI keeps its own; which of them is dialled when the start
/// names none; the port a stop is heard on; and how a daemon is
/// dialled again.
///
/// Nothing set is this type's [`Default`]. An unknown key is an
/// error, so a misspelled setting is refused rather than ignored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    /// The port, on the loopback only, the app hears `POST /stop` on
    /// for as long as it runs: how `diverge viewer stop` reaches it.
    pub port: u16,
    /// Whether the app dials the daemon again by itself when the
    /// connection ends or a dial fails, and how long it waits first:
    /// `never`, or a whole number of seconds, at least `1`. Absent,
    /// every 5 seconds, as the daemon redials a provider — see
    /// [`Redial`].
    #[serde(skip_serializing_if = "Redial::is_default")]
    pub redial: Redial,
    /// The daemon dialled when the start names none, by its name in
    /// `daemons`.
    pub default: String,
    /// The daemons the app may dial, by name, in the order they were
    /// added.
    pub daemons: IndexMap<String, Daemon>,
}

/// The built-in default: the stop heard on [`DEFAULT_PORT`], a redial
/// of five seconds, and one daemon, [`LOCAL`] — the daemon on this
/// host at a daemon's own default port, which is the CLI's default
/// daemon too — with no account. An account is an identity somebody
/// signed in, and none can be supposed: the app does not start until
/// the daemon it is to dial has one.
impl Default for Config {
    fn default() -> Self {
        let local = Daemon {
            address: format!("ws://127.0.0.1:{}/", diverge_sdk::daemon::DEFAULT_PORT),
            default: None,
            identity: None,
            accounts: IndexMap::new(),
        };
        Config {
            port: DEFAULT_PORT,
            redial: Redial::default(),
            default: LOCAL.to_string(),
            daemons: IndexMap::from([(LOCAL.to_string(), local)]),
        }
    }
}
