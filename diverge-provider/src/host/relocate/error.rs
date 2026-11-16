//! Why podman's data could not be carried.

use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::host::tools;

/// What a [`transfer`](super::transfer) fails with. Every one of them
/// leaves the data where it was.
#[derive(Debug)]
pub enum Error {
    /// A file or a folder could not be read, made, renamed, copied or
    /// removed.
    Io {
        /// Which.
        path: PathBuf,
        /// Why.
        source: io::Error,
    },
    /// A program the transfer runs — podman, `cp`, `df` — could not
    /// be started, or refused.
    Tool(tools::Error),
    /// One path is inside the other.
    Nested {
        /// The path the data is at.
        old: PathBuf,
        /// The path asked for.
        new: PathBuf,
    },
    /// The new path is there, holds something, and is not a transfer
    /// from the old path left unfinished.
    Occupied(PathBuf),
    /// An entry is at both the old path and the new, and neither is a
    /// folder to merge.
    Conflict(PathBuf),
    /// The new path's filesystem has not the room.
    Space {
        /// The new path.
        path: PathBuf,
        /// The bytes to carry.
        needed: u64,
        /// The bytes free there.
        free: u64,
    },
    /// A file copied is not the size of its source.
    Size(PathBuf),
    /// A link cannot be carried to another drive on Windows.
    Link(PathBuf),
    /// A path is not text, so no file can be searched for it.
    NotText(PathBuf),
    /// A file still names the old path after it was rewritten.
    Remains(PathBuf),
    /// On Linux: these are mounted at or under the old path.
    Mounted(Vec<PathBuf>),
    /// On Linux: the store under the old path holds containers, pods
    /// or volumes, and podman would not take it at another path.
    Held {
        /// How many containers.
        containers: usize,
        /// How many pods.
        pods: usize,
        /// How many volumes.
        volumes: usize,
    },
    /// On Windows: WSL would not move a distribution's disk, did not
    /// register it at the new place, or would not unregister it.
    Wsl {
        /// The distribution.
        distribution: String,
        /// What WSL said, or what was found.
        why: String,
    },
    /// On Windows: the registry's list of WSL distributions could not
    /// be read.
    Registry(String),
    /// On Windows: the connections file is not the JSON podman
    /// writes, so the connections to a deleted machine could not be
    /// taken out of it.
    Connections(PathBuf),
    /// On Windows, asked to leave the data behind: the machine podman
    /// keeps a description of names its SSH key under the old path, and
    /// the provider would refuse to start beside it.
    Blocked(PathBuf),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io { path, source } => write!(f, "`{}`: {source}", path.display()),
            Error::Tool(error) => write!(f, "{error}"),
            Error::Nested { old, new } => write!(
                f,
                "`{}` and `{}` are one inside the other, and podman's data cannot be moved into or out of itself",
                old.display(),
                new.display()
            ),
            Error::Occupied(path) => write!(f, "`{}` is there already and is not empty", path.display()),
            Error::Conflict(path) => write!(f, "`{}` is at both the old path and the new", path.display()),
            Error::Space { path, needed, free } => write!(
                f,
                "`{}` is on another filesystem, so podman's data is copied there, and it has {free} bytes free of the {needed} needed",
                path.display()
            ),
            Error::Size(path) => write!(f, "`{}` was copied and is not the size of its source", path.display()),
            Error::Link(path) => write!(f, "`{}` is a link, which cannot be carried to another drive", path.display()),
            Error::NotText(path) => write!(f, "`{}` is not text", path.display()),
            Error::Remains(path) => write!(f, "`{}` still names the old path", path.display()),
            Error::Mounted(mounts) => {
                write!(f, "these are still mounted under the old path, and must be unmounted first:")?;
                for mount in mounts {
                    write!(f, " `{}`", mount.display())?;
                }
                Ok(())
            }
            Error::Held { containers, pods, volumes } => write!(
                f,
                "podman's store holds {containers} containers, {pods} pods and {volumes} volumes, and podman takes a store at another path only when it holds none: remove them first"
            ),
            Error::Wsl { distribution, why } => write!(f, "the WSL distribution `{distribution}`: {why}"),
            Error::Registry(why) => write!(f, "the WSL distributions could not be read from the registry: {why}"),
            Error::Connections(path) => write!(f, "`{}` is not the connections file podman writes", path.display()),
            Error::Blocked(path) => write!(
                f,
                "podman still holds the machine whose data is under `{}`, and the provider would refuse to start beside it: transfer the data, or delete it first with `diverge provider podman storage_path delete`",
                path.display()
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io { source, .. } => Some(source),
            Error::Tool(error) => Some(error),
            _ => None,
        }
    }
}

impl From<tools::Error> for Error {
    fn from(error: tools::Error) -> Self {
        Error::Tool(error)
    }
}

impl Error {
    /// An I/O failure at `path`.
    pub(super) fn io(path: &std::path::Path, source: io::Error) -> Self {
        Error::Io { path: path.to_path_buf(), source }
    }
}
