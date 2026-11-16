//! Podman's data carried from one `podman.storage_path` to another.
//!
//! The storage path is where the provider's podman keeps everything:
//! on Linux the image store itself, on macOS and Windows a whole
//! machine, its disk tens of gigabytes. Naming another path in the
//! configuration leaves all of it behind. [`transfer`] carries it
//! instead, with the provider stopped: it is what the CLI's
//! `provider podman storage_path set` does before it writes the new
//! path, and the provider itself never calls it.
//!
//! # Why a folder moved is not enough
//!
//! What lies under the path is pointed at, by absolute path, from
//! beside it and from outside it:
//!
//! - **Linux.** The path is podman's graphroot. Its database records
//!   the graphroot, and podman rewrites the record by itself only
//!   while the database holds no container, pod or volume; it refuses
//!   to run otherwise, and a container's own record holds more
//!   absolute paths besides. So a store that holds any is refused,
//!   as is one with a mount still under it — see [`linux`]. Nothing
//!   else names the path: the layers' links are relative.
//! - **macOS.** Podman is given the path as both its data home and
//!   its configuration home, so the machine's disk, EFI store, SSH key,
//!   description and connections all lie under it and move with it;
//!   but the description and the connections name the disk, the EFI
//!   store and the key by absolute path.
//! - **Windows.** Podman keeps a machine's description under the
//!   user's `.config` and its connections under `%APPDATA%`, wherever
//!   its data home is, and both name the SSH key and the image under
//!   the path. And the machine's disk is a WSL distribution's, whose
//!   place WSL keeps in the registry — see [`wsl`].
//!
//! # The rules
//!
//! 1. **Nothing to carry.** An old path that is not there, or is an
//!    empty folder, is nothing to carry.
//! 2. **Apart.** The new path inside the old, or the old inside the
//!    new, is refused.
//! 3. **The new path is free.** It is not there, or is an empty
//!    folder, or holds the marker of a transfer from this same old
//!    path that did not finish — rule 8.
//! 4. **Podman is still first.** On macOS and Windows the machine
//!    under the old path is stopped if it runs, whoever left it
//!    running; on Windows every WSL distribution whose disk is under
//!    the old path is terminated. On Linux a mount under the path, or
//!    a container, pod or volume in the store, is refused.
//! 5. **A move is a rename.** Where the two paths are on one
//!    filesystem every entry is renamed and no byte is copied, however
//!    large. Whether they are is asked of the filesystem, by renaming
//!    a probe, and not guessed from the paths.
//! 6. **Across filesystems it is a copy, checked, and the source
//!    deleted last.** There is no other way to cross one. Room for the
//!    whole is asked for first; every file's size is compared after;
//!    and the source is deleted by [`Transfer::finish`], which the
//!    caller calls once the new path is written to the configuration.
//!    The one exception is a WSL disk, which WSL itself moves, and
//!    moves back if the transfer is undone.
//! 7. **All or nothing.** Every step is recorded as it is done. A
//!    step that fails undoes those before it, last first, and the
//!    error is returned with everything where it was;
//!    [`Transfer::undo`] is the same for a caller whose write of the
//!    configuration failed.
//! 8. **It can be run again.** A marker in the new folder names the
//!    old path for as long as a transfer is under way. Every step
//!    looks at what is so before it acts — an entry already carried, a
//!    distribution already at the new place, a file that names the old
//!    path no longer — so a transfer cut short by the host going down
//!    is finished by asking for the same one again.
//! 9. **Checked before it is called done.** No file that named the
//!    old path names it still, and on Windows every distribution
//!    carried is registered at the new place.
//!
//! Its own files are flattened into it, so everything is named
//! through this module and not through the file it lives in.

mod error;
mod moving;
mod rewrite;
mod space;
mod transfer;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(not(target_os = "linux"))]
pub mod machine;
#[cfg(windows)]
pub mod wsl;

pub use error::*;
pub use transfer::*;
