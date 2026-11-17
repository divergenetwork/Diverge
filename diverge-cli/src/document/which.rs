//! Which document, and its type.

use serde_json::Value;

use super::Error;

/// One of the four documents in the root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Which {
    /// The daemon's.
    Daemon,
    /// The provider's.
    Provider,
    /// The viewer's.
    Viewer,
    /// The CLI's own.
    Cli,
}

impl Which {
    /// The file's name in the root.
    pub fn file(self) -> &'static str {
        match self {
            Which::Daemon => "daemon.yaml",
            Which::Provider => "provider.yaml",
            Which::Viewer => "viewer.yaml",
            Which::Cli => "cli.yaml",
        }
    }

    /// The document as its program holds it when there is no file:
    /// the type's built-in default, as JSON.
    pub fn default(self) -> Result<Value, Error> {
        let value = match self {
            Which::Daemon => serde_json::to_value(diverge_daemon::config::Config::default()),
            Which::Provider => serde_json::to_value(diverge_provider::config::Config::default()),
            Which::Viewer => serde_json::to_value(crate::config::Viewer::default()),
            Which::Cli => serde_json::to_value(crate::config::Config::default()),
        };
        value.map_err(Error::Json)
    }
}
