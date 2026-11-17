//! What `diverge viewer` takes.

use clap::Subcommand;
use diverge_sdk::wire::proxy::Redial;

use super::{Chosen, Daemons, Port};

/// The viewer on this host: started, stopped, or one of its settings
/// — the port its stop is heard on, its redial, its daemons, what
/// each of them is, and which of them is the default — read or
/// written by itself. Its daemons are kept as the CLI's own are. A
/// setting is not written while the viewer runs on the root.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Viewer {
    /// Start the viewer on the root, detached, on one of its daemons
    /// as one of that daemon's accounts; the command ends once the
    /// viewer is up, and fails if one already runs on the root or it
    /// dies before it is up, with what it said. The window's title
    /// names the daemon.
    Start {
        /// The daemon to dial, by its name among the viewer's
        /// daemons. Absent, the viewer's default daemon.
        #[arg(long, value_name = "NAME")]
        daemon: Option<String>,
        /// The account to dial it as, by its name among that daemon's
        /// accounts. Absent, the daemon's default account.
        #[arg(long, value_name = "NAME")]
        account: Option<String>,
    },
    /// Stop the viewer running on the root, asked over its own port
    /// on this host; the command ends once the viewer has exited.
    Stop,
    /// Read or write the loopback port the viewer hears its stop on.
    #[command(subcommand)]
    Port(Port),
    /// Read or write whether, and how long after, the viewer dials
    /// its daemon again when the connection ends.
    #[command(subcommand)]
    Redial(RedialSetting),
    /// List the daemons the viewer may dial, add one, delete one, or
    /// — after a daemon's index — read or write one setting of that
    /// daemon.
    Daemons(Daemons),
    /// Read or write which daemon is dialled when a start names none.
    #[command(subcommand)]
    Default(Chosen),
}

/// What is done to a setting that is a string: a daemon's address.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Text {
    /// Print the string, alone on its line, as it is and not as
    /// JSON.
    Get,
    /// Write the string.
    Set {
        /// The string.
        #[arg(value_name = "VALUE")]
        value: String,
    },
}

/// What is done to the viewer's `redial`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum RedialSetting {
    /// Print `never`, or the number of seconds: `5` where none was
    /// set.
    Get,
    /// Write `never`, or a whole number of seconds. Refused while
    /// the viewer is running on the root: stop it first.
    Set {
        /// `never`, or the seconds to wait before dialling again: at
        /// least `1`.
        #[arg(value_name = "never|SECONDS", value_parser = redial)]
        value: Redial,
    },
}

/// A redial from the command line: the word, or the seconds, read as
/// the viewer reads them.
fn redial(given: &str) -> Result<Redial, String> {
    let value = match given.parse::<u64>() {
        Ok(seconds) => serde_json::Value::from(seconds),
        Err(_) => serde_json::Value::from(given),
    };
    serde_json::from_value(value).map_err(|_| "`never`, or a whole number of seconds, at least 1".to_string())
}
