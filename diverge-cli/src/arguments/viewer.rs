//! What `diverge viewer` takes.

use clap::Subcommand;
use diverge_sdk::wire::proxy::Redial;

use super::Identity;

/// The viewer's document in the root, `viewer.yaml`: one setting of
/// its `daemon` read or written by itself. The document is not worked
/// on whole: there is no `viewer config`. The viewer is not started
/// or stopped here, and holds no lock on the root: a setting is
/// written whether the app is open or not, and read by the app at its
/// next start.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Viewer {
    /// Read or write one setting of the daemon the viewer dials, and
    /// as whom, `daemon` in `viewer.yaml`.
    #[command(subcommand)]
    Daemon(ViewerDaemon),
}

/// The settings of the viewer's `daemon`, each a command beneath
/// `diverge viewer daemon`, named as the document names it. Every one
/// has a default, so every one is read and written whether the
/// document names a daemon or not.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum ViewerDaemon {
    /// Read or write where the daemon is dialled, `daemon.address`.
    #[command(subcommand)]
    Address(Text),
    /// Read or write the issuer of the identity the viewer dials as,
    /// `daemon.issuer`.
    #[command(subcommand)]
    Issuer(Text),
    /// Read or write the subject of the identity the viewer dials as,
    /// `daemon.subject`.
    #[command(subcommand)]
    Subject(Text),
    /// Read or write the daemon's own identity, which must answer at
    /// the address where it is stated, `daemon.identity`.
    #[command(subcommand)]
    Identity(Identity),
    /// Read or write whether, and how often, the viewer dials the
    /// daemon again by itself, `daemon.redial`.
    #[command(subcommand)]
    Redial(RedialSetting),
}

/// What is done to a setting that is a string.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Text {
    /// Print the string, alone on its line, as it is and not as
    /// JSON: the default where the document has none.
    Get,
    /// Write the string.
    Set {
        /// The string, as it is to be written.
        #[arg(value_name = "VALUE")]
        value: String,
    },
}

/// What is done to the viewer's `daemon.redial`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum RedialSetting {
    /// Print `never`, or the number of seconds: `5` where the
    /// document has none.
    Get,
    /// Write `never`, or a whole number of seconds.
    Set {
        /// `never`, or a whole number of seconds, `1` or more.
        #[arg(value_name = "never|SECONDS", value_parser = redial)]
        value: Redial,
    },
}

/// A redial from the command line: the word, or the seconds, read as
/// the document reads them.
fn redial(given: &str) -> Result<Redial, String> {
    let value = match given.parse::<u64>() {
        Ok(seconds) => serde_json::Value::from(seconds),
        Err(_) => serde_json::Value::String(given.to_string()),
    };
    serde_json::from_value(value).map_err(|_| "`never`, or a whole number of seconds, at least 1".to_string())
}
