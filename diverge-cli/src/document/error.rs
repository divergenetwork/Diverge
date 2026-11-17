//! Why a document could not be read or written.

use std::fmt;
use std::io;
use std::path::PathBuf;

/// What the document operations fail with: the filesystem, the file's
/// own content, or the type's refusal.
#[derive(Debug)]
pub enum Error {
    /// A file could not be read, written, renamed or removed.
    Io {
        /// What could not be.
        path: PathBuf,
        /// Why.
        source: io::Error,
    },
    /// A file is not YAML.
    Parse {
        /// The file.
        path: PathBuf,
        /// Where in it, and what was wrong.
        source: serde_path_to_error::Error<serde_yaml_ng::Error>,
    },
    /// The value is not one the document's type takes: a key it does
    /// not have, a value of the wrong kind. Nothing was written.
    Invalid(serde_path_to_error::Error<serde_json::Error>),
    /// The type's own default could not be written as JSON.
    Json(serde_json::Error),
    /// The typed document could not be written as YAML.
    Serialize(serde_yaml_ng::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io { path, source } => {
                write!(f, "`{}` could not be used: {source}", path.display())
            }
            Error::Parse { path, source } => {
                write!(f, "`{}` could not be read: {source}", path.display())
            }
            Error::Invalid(error) => {
                write!(f, "the settings are not ones their program takes: {error}")
            }
            Error::Json(error) => write!(
                f,
                "the default settings could not be written as JSON: {error}"
            ),
            Error::Serialize(error) => {
                write!(f, "the settings could not be written: {error}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io { source, .. } => Some(source),
            Error::Parse { source, .. } => Some(source),
            Error::Invalid(error) => Some(error),
            Error::Json(error) => Some(error),
            Error::Serialize(error) => Some(error),
        }
    }
}
