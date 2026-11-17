//! Why the program could not run.

use std::fmt;
use std::io;

use diverge_sdk::identities;
use diverge_sdk::identity_diverge_network;

use crate::config;
use crate::detach;
use crate::document;
use crate::lifecycle;
use crate::lifecycle::Program;
use diverge_provider::host::relocate;
use crate::ours;
use crate::run;

/// What [`run`](super::run) fails with: the runtime, the
/// configuration, a document, a request, the identities, or a
/// program's start or stop. A `main` returning one of these prints it, and that
/// is the one report a failed run gets, so its `Debug` is its
/// `Display`.
pub enum Error {
    /// The runtime could not be built.
    Runtime(io::Error),
    /// The root, the file, or the names could not be used.
    Config(config::Error),
    /// A document could not be read, rewritten or edited.
    Document(document::Error),
    /// A request could not be put, or its answers printed.
    Run(run::Error),
    /// The daemon or the provider could not be started or stopped.
    Lifecycle(lifecycle::Error),
    /// The root's identities refused, could not be opened, or a login
    /// did not come to an identity.
    Identities(identities::Error),
    /// Stdout could not be written.
    Stdout(io::Error),
    /// A command that leaves its wait behind could not be relayed, or
    /// ended before it left.
    Detach(detach::Error),
    /// The root's accounts at `identity.diverge.network` could not be
    /// opened or listed, a login could not begin, or a request made as
    /// one was refused.
    Accounts(identity_diverge_network::Error),
    /// A login at `identity.diverge.network` could not be made here.
    Ours(ours::Error),
    /// A setting was not written, because the program whose document
    /// it is in is running on the root: the program, and the process
    /// id it left in the lock file, where it left one.
    Stop {
        /// Which program.
        program: Program,
        /// What its holder wrote in the lock file.
        holder: Option<String>,
    },
    /// A setting was not written, because the section it is in is
    /// absent from the document: the feature is off.
    Disabled {
        /// The section, as the document names it: `containers`,
        /// `volumes`.
        section: &'static str,
        /// The program whose document it is, as the CLI names it.
        program: Program,
    },
    /// Podman's data, or a volume store, could not be moved to its
    /// new path. Nothing was moved, and nothing was written.
    Relocate(relocate::Error),
    /// Podman's data, or a volume store, is at its new path and the
    /// path is written, but what was left at the old path could not
    /// be removed.
    Leftover(relocate::Error),
    /// Podman's data at the storage path, or a volume store, could
    /// not be deleted, or not all of it.
    Wipe(relocate::Error),
    /// A store's capacity was not written, because the volumes in
    /// the store reserve more than it.
    Capacity {
        /// The store's index.
        index: usize,
        /// The capacity asked for, in bytes.
        capacity: u64,
        /// What the volumes in the store reserve, in bytes.
        used: u64,
    },
    /// A path could not be made absolute.
    Path(std::io::Error),
    /// A list of a document has nothing at the index a delete named.
    /// Nothing was written.
    NoItem {
        /// The list, as the document names it: `admins`, `incoming`,
        /// `outgoing`, `podman.registries`, `volumes.fixed`,
        /// `volumes.stores`.
        list: &'static str,
        /// The index named.
        index: usize,
        /// How many items the list has.
        count: usize,
    },
    /// The item an add named is in the list already. Nothing was
    /// written.
    Listed {
        /// The list, as the document names it.
        list: &'static str,
        /// The item, as an error says it.
        item: String,
        /// Where in the list it is.
        index: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Runtime(error) => write!(f, "the runtime could not be built: {error}"),
            Error::Config(error) => write!(f, "{error}"),
            Error::Document(error) => write!(f, "{error}"),
            Error::Run(error) => write!(f, "{error}"),
            Error::Lifecycle(error) => write!(f, "{error}"),
            Error::Identities(error) => write!(f, "{error}"),
            Error::Stdout(error) => write!(f, "stdout could not be written: {error}"),
            Error::Detach(error) => write!(f, "{error}"),
            Error::Accounts(error) => write!(f, "{error}"),
            Error::Ours(error) => write!(f, "{error}"),
            Error::Stop { program, holder } => {
                let word = program.word();
                write!(f, "the {word} is running on this root")?;
                if let Some(holder) = holder {
                    write!(f, ", as process {holder}")?;
                }
                write!(f, ", and nothing was written: stop it first, with `diverge {word} stop`")
            }
            Error::Disabled { section, program } => write!(
                f,
                "`{section}` is disabled, and nothing was written: enable it first, with `diverge {} {section} enable`",
                program.word()
            ),
            Error::Relocate(error) => write!(f, "the data was not moved, and the path was not written: {error}"),
            Error::Leftover(error) => write!(
                f,
                "the data is at the new path and the path is written, but the old path could not be cleared: {error}"
            ),
            Error::Wipe(error) => write!(f, "the data was not deleted, or not all of it, and nothing was written; asked again, the rest is: {error}"),
            Error::Capacity { index, capacity, used } => write!(
                f,
                "the volumes in store {index} reserve {used} bytes, and its capacity cannot be {capacity}, which is less; nothing was written"
            ),
            Error::Path(error) => write!(f, "the path could not be made absolute: {error}"),
            Error::NoItem { list, index, count: 0 } => {
                write!(f, "`{list}` has nothing at index {index}: it is empty")
            }
            Error::NoItem { list, index, count } => write!(
                f,
                "`{list}` has nothing at index {index}: it has {count}, the last at index {}",
                count - 1
            ),
            Error::Listed { list, item, index } => {
                write!(f, "{item} is in `{list}` already, at index {index}")
            }
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
            Error::Runtime(error) => Some(error),
            Error::Config(error) => Some(error),
            Error::Document(error) => Some(error),
            Error::Run(error) => Some(error),
            Error::Lifecycle(error) => Some(error),
            Error::Identities(error) => Some(error),
            Error::Stdout(error) => Some(error),
            Error::Detach(error) => Some(error),
            Error::Accounts(error) => Some(error),
            Error::Ours(error) => Some(error),
            Error::Relocate(error) | Error::Leftover(error) | Error::Wipe(error) => Some(error),
            Error::Path(error) => Some(error),
            Error::Stop { .. } | Error::NoItem { .. } | Error::Listed { .. } | Error::Disabled { .. } | Error::Capacity { .. } => None,
        }
    }
}

impl From<config::Error> for Error {
    fn from(error: config::Error) -> Self {
        Error::Config(error)
    }
}

impl From<identity_diverge_network::Error> for Error {
    fn from(error: identity_diverge_network::Error) -> Self {
        Error::Accounts(error)
    }
}

impl From<ours::Error> for Error {
    fn from(error: ours::Error) -> Self {
        Error::Ours(error)
    }
}

impl From<detach::Error> for Error {
    fn from(error: detach::Error) -> Self {
        Error::Detach(error)
    }
}

impl From<document::Error> for Error {
    fn from(error: document::Error) -> Self {
        Error::Document(error)
    }
}

impl From<run::Error> for Error {
    fn from(error: run::Error) -> Self {
        Error::Run(error)
    }
}

impl From<identities::Error> for Error {
    fn from(error: identities::Error) -> Self {
        Error::Identities(error)
    }
}

impl From<lifecycle::Error> for Error {
    fn from(error: lifecycle::Error) -> Self {
        Error::Lifecycle(error)
    }
}
