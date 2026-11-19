//! Where an ephemeral FUSE serve's scratch lives, how one is made, and
//! how the leftovers of an earlier life are swept.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::fs;

use super::sparse;
use crate::host::{Held, Limit};

/// Where every ephemeral layer the provider keeps itself is: a FUSE
/// serve's, and a container's ephemeral mount's, each a file of its
/// own. It is `<podman.storage_path>/ephemeral/`, on every provider,
/// whether it runs containers or not.
pub fn scratch_dir(storage_path: &Path) -> PathBuf {
    storage_path.join("ephemeral")
}

/// The scratch directory and the cap the scratch counts against,
/// shared by every stored volume.
///
/// The directory is [`scratch_dir`], and the cap is
/// `podman.overlay_disk`: the one [`Limit`] the running containers'
/// `disk` and their ephemeral mounts' `overlay_disk` are taken from
/// too, so that a FUSE serve's `overlay_disk` is counted beside
/// them.
#[derive(Debug)]
pub struct Scratch {
    dir: PathBuf,
    disk: Arc<Limit>,
}

/// Bytes held against the cap for one FUSE serve; given back on drop,
/// since that is what a [`Held`] does.
#[derive(Debug)]
pub struct Lease {
    _held: Held,
}

impl Scratch {
    pub fn new(dir: PathBuf, disk: Arc<Limit>) -> Self {
        Scratch { dir, disk }
    }

    /// The directory made, and every leftover in it removed: what the
    /// provider does once at start, before any FUSE serve. A file that
    /// will not go is left, as a mount that will not unmount is.
    pub async fn sweep(&self) -> io::Result<()> {
        fs::create_dir_all(&self.dir).await?;
        let mut entries = fs::read_dir(&self.dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let _ = fs::remove_file(entry.path()).await;
        }
        Ok(())
    }

    /// `overlay_disk` taken from the cap, or `None` with nothing
    /// taken.
    pub fn lease(&self, overlay_disk: u64) -> Option<Lease> {
        self.disk.take(overlay_disk).map(|held| Lease { _held: held })
    }

    /// A fresh scratch file, handed over as a std handle for the
    /// blocking closure that will own it: made under a v4 UUID name,
    /// marked sparse, and made to vanish when its handle closes — on
    /// Unix unlinked at once, the inode living with the handle; on
    /// Windows opened to delete on close — so its removal is the
    /// host's and nothing here removes it on drop.
    pub async fn create(&self) -> io::Result<std::fs::File> {
        let path = self.dir.join(uuid::Uuid::new_v4().to_string());
        let mut options = fs::OpenOptions::new();
        options.read(true).write(true).create_new(true);
        #[cfg(windows)]
        options.custom_flags(windows_sys::Win32::Storage::FileSystem::FILE_FLAG_DELETE_ON_CLOSE);
        let file = options.open(&path).await?;
        sparse::sparse(&file).await?;
        #[cfg(not(windows))]
        fs::remove_file(&path).await?;
        Ok(file.into_std().await)
    }
}
