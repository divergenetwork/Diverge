//! Why the daemon's state could not be made whole.

use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::logs;
use crate::store;

/// What [`run`](super::run) fails with: the store, a log, a folder,
/// or a log's own `deployed`.
pub enum Error {
    /// The store refused.
    Store(store::Error),
    /// A log's file could not be read, cut or appended to.
    Logs(logs::Error),
    /// A folder of logs could not be listed, or one in it removed.
    Folder {
        /// Which.
        path: PathBuf,
        /// Why.
        source: io::Error,
    },
    /// A log's latest `deployed` names no provider that can be read,
    /// so its `undeployed` cannot be written.
    Deployed(PathBuf),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Store(error) => write!(f, "{error}"),
            Error::Logs(error) => write!(f, "{error}"),
            Error::Folder { path, source } => write!(f, "{}: {source}", path.display()),
            Error::Deployed(path) => write!(f, "{}: its latest `deployed` names no provider", path.display()),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Store(error) => Some(error),
            Error::Logs(error) => Some(error),
            Error::Folder { source, .. } => Some(source),
            Error::Deployed(_) => None,
        }
    }
}

impl From<store::Error> for Error {
    fn from(error: store::Error) -> Self {
        Error::Store(error)
    }
}

impl From<logs::Error> for Error {
    fn from(error: logs::Error) -> Self {
        Error::Logs(error)
    }
}
