//! Why the root or the file could not be used.

use std::fmt;
use std::io;
use std::path::PathBuf;

/// What [`arguments`](super::arguments), [`root`](super::root),
/// [`load`](super::load) and [`select`](super::select) fail with: the
/// arguments, the host, the filesystem, the file's own content, or a
/// daemon or an account the settings do not have.
#[derive(Debug)]
pub enum Error {
    /// An argument this program does not take, one given twice, or
    /// one with nothing after it.
    Arguments(String),
    /// No root was named and the host has no home directory to put
    /// one under.
    Home,
    /// The root could not be made, or the file could not be read.
    Io {
        /// What could not be.
        path: PathBuf,
        /// Why.
        source: io::Error,
    },
    /// The file does not parse, or says something `viewer.yaml` does not
    /// take.
    Parse {
        /// The file.
        path: PathBuf,
        /// Where in it, and what was wrong.
        source: serde_path_to_error::Error<serde_yaml_ng::Error>,
    },
    /// No daemon of the settings has the name.
    NoDaemon(String),
    /// No account was named, and the daemon has no default.
    NoDefaultAccount(String),
    /// No account of the daemon has the name.
    NoAccount {
        /// The daemon.
        daemon: String,
        /// The account.
        account: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Arguments(argument) => {
                write!(f, "the argument `{argument}` is not one this program takes once, with a value after it; `--config <dir>`, `--daemon <name>` and `--account <name>` are")
            }
            Error::Home => write!(f, "no root was named and the host has no home directory"),
            Error::Io { path, source } => write!(f, "`{}` could not be used: {source}", path.display()),
            Error::Parse { path, source } => write!(f, "`{}` could not be read: {source}", path.display()),
            Error::NoDaemon(name) => write!(f, "the viewer has no daemon named `{name}`: `diverge viewer daemons get` lists them"),
            Error::NoDefaultAccount(daemon) => write!(
                f,
                "no account was named, and the viewer's daemon `{daemon}` has no default: sign an identity in with `diverge identities login`, add it with `diverge viewer daemons <INDEX> accounts add`, and make it the default with `diverge viewer daemons <INDEX> default set <INDEX>`"
            ),
            Error::NoAccount { daemon, account } => write!(f, "the viewer's daemon `{daemon}` has no account named `{account}`"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Arguments(_) | Error::Home | Error::NoDaemon(_) | Error::NoDefaultAccount(_) | Error::NoAccount { .. } => None,
            Error::Io { source, .. } => Some(source),
            Error::Parse { source, .. } => Some(source),
        }
    }
}
