//! Why an update could not be made.

use std::fmt;
use std::io;
use std::path::PathBuf;

/// What [`update`](super::update) fails with. Each says whether the
/// installed programs were changed: nothing but [`Swap`](Self::Swap)
/// whose `restored` is `false` leaves them other than they were.
#[derive(Debug)]
pub enum Error {
    /// The releases could not be asked, or an archive not fetched.
    Http {
        /// The address asked.
        url: String,
        /// Why.
        source: reqwest::Error,
    },
    /// The releases answered something other than success.
    Status {
        /// The address asked.
        url: String,
        /// The status answered.
        status: u16,
    },
    /// There is no such version.
    NoVersion(String),
    /// The version was not built for the platform.
    NoPlatform {
        /// The version.
        version: String,
        /// The operating system.
        os: String,
        /// The architecture.
        arch: String,
    },
    /// The version's release holds no skills: nothing was changed.
    NoSkills(String),
    /// A file or a folder could not be read, written, made or
    /// removed.
    Io {
        /// Which.
        path: PathBuf,
        /// Why.
        source: io::Error,
    },
    /// The archive could not be read as the archive it is named as.
    Archive(String),
    /// The archive does not hold a program every release holds.
    /// Nothing was replaced.
    Incomplete(String),
    /// A program could not be renamed into place.
    Swap {
        /// The program's file.
        file: String,
        /// Why.
        source: io::Error,
        /// Whether every program already replaced was put back: the
        /// installed programs are as they were.
        restored: bool,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Http { url, source } => write!(f, "{url} could not be fetched: {source}"),
            Error::Status { url, status } => write!(f, "{url} answered {status}"),
            Error::NoVersion(version) => write!(f, "there is no version {version}"),
            Error::NoPlatform { version, os, arch } => write!(f, "version {version} was not built for {os}/{arch}"),
            Error::NoSkills(version) => write!(f, "version {version} was released with no skills; nothing was changed — update with `--skip-updating-skills`"),
            Error::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Error::Archive(reason) => write!(f, "the archive could not be read: {reason}"),
            Error::Incomplete(file) => write!(f, "the archive does not hold {file}; nothing was replaced"),
            Error::Swap { file, source, restored: true } => write!(f, "{file} could not be replaced: {source}; every program is as it was"),
            Error::Swap { file, source, restored: false } => {
                write!(f, "{file} could not be replaced: {source}; and not every program could be put back — run the update again")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Http { source, .. } => Some(source),
            Error::Io { source, .. } | Error::Swap { source, .. } => Some(source),
            Error::Status { .. } | Error::NoVersion(_) | Error::NoPlatform { .. } | Error::NoSkills(_) | Error::Archive(_) | Error::Incomplete(_) => None,
        }
    }
}
