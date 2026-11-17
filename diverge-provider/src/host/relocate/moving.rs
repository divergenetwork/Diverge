//! Entries carried from one folder to another: renamed, or copied.

use std::future::Future;
use std::io;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use tokio::io::{AsyncReadExt as _, AsyncSeekExt as _, AsyncWriteExt as _};

use super::{Error, Step};

/// The file a transfer leaves in the new folder while it is under
/// way, holding the old path: rule 8 of [`relocate`](super).
pub(super) const MARKER: &str = ".diverge-relocating";

/// The file renamed from the old folder to the new to ask whether
/// the two are on one filesystem.
const PROBE: &str = ".diverge-relocate-probe";

/// How entries cross from the old folder to the new.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Mode {
    /// The two are on one filesystem: an entry is renamed.
    Rename,
    /// They are not: an entry is copied, and its source deleted when
    /// the transfer is finished.
    Copy,
}

/// How entries cross from `old` to `new`, both of them folders that
/// are there: asked of the filesystem, by making a file in `old` and
/// renaming it into `new`. A rename refused as crossing devices is
/// [`Mode::Copy`]; one that goes through is [`Mode::Rename`]; any
/// other refusal is the error, since it would refuse the entries too.
/// The probe is removed either way.
pub(super) async fn mode(old: &Path, new: &Path) -> Result<Mode, Error> {
    let from = old.join(PROBE);
    let to = new.join(PROBE);
    tokio::fs::write(&from, b"").await.map_err(|source| Error::io(&from, source))?;
    match tokio::fs::rename(&from, &to).await {
        Ok(()) => {
            tokio::fs::remove_file(&to).await.map_err(|source| Error::io(&to, source))?;
            Ok(Mode::Rename)
        }
        Err(error) => {
            let _ = tokio::fs::remove_file(&from).await;
            if crosses(&error) { Ok(Mode::Copy) } else { Err(Error::io(&to, error)) }
        }
    }
}

/// Whether a rename was refused for crossing filesystems: `EXDEV` on
/// Unix, `ERROR_NOT_SAME_DEVICE` on Windows.
fn crosses(error: &io::Error) -> bool {
    let code = if cfg!(windows) { 17 } else { 18 };
    error.kind() == io::ErrorKind::CrossesDevices || error.raw_os_error() == Some(code)
}

/// Every entry of the folder `from` carried into the folder `to`,
/// each step recorded in `steps` as it is done. The rules:
///
/// - An entry with nothing of its name in `to` is carried whole: by
///   one rename, or by a copy.
/// - A folder with a folder of its name in `to` is merged into it,
///   entry by entry. That is how a folder made ahead in `to` — the
///   one WSL moved a disk into — takes the rest of its old self.
/// - Anything else at both places is [`Error::Conflict`], unless the
///   transfer is one `resumed`: then what is in `to` is the leaving
///   of a copy cut short, and is removed and carried again from
///   `from`, which a copy never touches.
/// - The marker and the probe are no entry of podman's, and are
///   passed over.
/// - A folder emptied by a merge is left where it is, empty, so that
///   an undo has where to put things back; the finish removes it.
pub(super) fn carry<'a>(
    mode: Mode,
    from: &'a Path,
    to: &'a Path,
    resumed: bool,
    steps: &'a mut Vec<Step>,
) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>> {
    Box::pin(async move {
        let mut listed = tokio::fs::read_dir(from).await.map_err(|source| Error::io(from, source))?;
        loop {
            let entry = listed.next_entry().await.map_err(|source| Error::io(from, source))?;
            let Some(entry) = entry else {
                break;
            };
            let name = entry.file_name();
            if name == MARKER || name == PROBE {
                continue;
            }
            let source = entry.path();
            let target = to.join(&name);
            let is_dir = folder(&source).await?;
            match tokio::fs::symlink_metadata(&target).await {
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(Error::io(&target, error)),
                Ok(meta) if meta.is_dir() && is_dir => {
                    carry(mode, &source, &target, resumed, steps).await?;
                    continue;
                }
                Ok(_) if !resumed => return Err(Error::Conflict(target)),
                Ok(_) => remove(&target).await?,
            }
            match mode {
                Mode::Rename => {
                    tokio::fs::rename(&source, &target).await.map_err(|error| Error::io(&source, error))?;
                    steps.push(Step::Renamed { from: source, to: target });
                }
                Mode::Copy => {
                    // Recorded before the copy, so that a copy that
                    // fails midway is removed by the undo.
                    steps.push(Step::Copied { from: source.clone(), to: target.clone() });
                    copy(&source, &target).await?;
                    same(&source, &target).await?;
                }
            }
        }
        Ok(())
    })
}

/// Whether `path` is a folder itself, and not a link to one.
async fn folder(path: &Path) -> Result<bool, Error> {
    let meta = tokio::fs::symlink_metadata(path).await.map_err(|source| Error::io(path, source))?;
    Ok(meta.is_dir())
}

/// `path` removed, whatever it is; one not there is nothing to
/// remove.
pub async fn remove(path: &Path) -> Result<(), Error> {
    let removed = match tokio::fs::symlink_metadata(path).await {
        Ok(meta) if meta.is_dir() => tokio::fs::remove_dir_all(path).await,
        Ok(_) => tokio::fs::remove_file(path).await,
        Err(error) => Err(error),
    };
    match removed {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(Error::io(path, source)),
    }
}

/// `from` copied to `to`, which is not there, across filesystems.
///
/// On Linux the store is root's, and holds what only `cp -a` carries
/// whole: owners, extended attributes — the overlay's among them —
/// device files, hard links; `--sparse=always` keeps the holes. On
/// macOS and Windows the tree is plain files and folders of the
/// user's own, and is copied here: a folder made and its entries
/// copied, a file copied with its holes kept and its permissions set
/// — the machine's SSH key is refused by `ssh` under any wider — and,
/// on macOS, a link made again as it was. A link on Windows is
/// [`Error::Link`]: podman's data holds none there.
async fn copy(from: &Path, to: &Path) -> Result<(), Error> {
    if cfg!(target_os = "linux") {
        let mut command = tokio::process::Command::new("cp");
        command.arg("-a").arg("--sparse=always").arg(from).arg(to);
        return Ok(crate::host::tools::run("cp", command).await?.require("cp", |status| status.success())?);
    }
    tree(from, to).await
}

/// The copy of [`copy`] on the hosts with a machine.
fn tree<'a>(from: &'a Path, to: &'a Path) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>> {
    Box::pin(async move {
        let meta = tokio::fs::symlink_metadata(from).await.map_err(|source| Error::io(from, source))?;
        if meta.is_symlink() {
            return link(from, to).await;
        }
        if meta.is_dir() {
            tokio::fs::create_dir(to).await.map_err(|source| Error::io(to, source))?;
            let mut listed = tokio::fs::read_dir(from).await.map_err(|source| Error::io(from, source))?;
            loop {
                let entry = listed.next_entry().await.map_err(|source| Error::io(from, source))?;
                let Some(entry) = entry else {
                    break;
                };
                tree(&entry.path(), &to.join(entry.file_name())).await?;
            }
        } else {
            sparse(from, to, meta.len()).await?;
        }
        tokio::fs::set_permissions(to, meta.permissions()).await.map_err(|source| Error::io(to, source))
    })
}

/// A link made at `to` as the one at `from` is.
#[cfg(unix)]
async fn link(from: &Path, to: &Path) -> Result<(), Error> {
    let target = tokio::fs::read_link(from).await.map_err(|source| Error::io(from, source))?;
    tokio::fs::symlink(&target, to).await.map_err(|source| Error::io(to, source))
}

/// A link is not carried across drives on Windows.
#[cfg(not(unix))]
async fn link(from: &Path, _to: &Path) -> Result<(), Error> {
    Err(Error::Link(from.to_path_buf()))
}

/// The bytes read and written at once: a megabyte.
const BLOCK: usize = 1024 * 1024;

/// The file `from`, `length` bytes long, copied to `to` with its
/// holes kept: a block that is all zeros is not written but stepped
/// over, and the file is set to its length at the end, so that a
/// machine's disk that is mostly empty takes at the new place what it
/// took at the old, where the filesystem keeps holes. The copy is
/// synced before it is called done, since its source is deleted on
/// the strength of it.
async fn sparse(from: &Path, to: &Path, length: u64) -> Result<(), Error> {
    let mut source = tokio::fs::File::open(from).await.map_err(|error| Error::io(from, error))?;
    let mut target = tokio::fs::File::create(to).await.map_err(|error| Error::io(to, error))?;
    // On NTFS a file keeps holes only once it is marked sparse, and a
    // volume's image is one: tens of gigabytes reserved, few taken.
    // The copy of a file that is marked is marked before it is
    // written, or it would take at the new place all that it reserves.
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt as _;
        /// `FILE_ATTRIBUTE_SPARSE_FILE`.
        const SPARSE: u32 = 0x200;
        let attributes = source.metadata().await.map_err(|error| Error::io(from, error))?.file_attributes();
        if attributes & SPARSE != 0 {
            crate::host::volume_manager::sparse(&target).await.map_err(|error| Error::io(to, error))?;
        }
    }
    let mut block = vec![0u8; BLOCK];
    loop {
        let read = source.read(&mut block).await.map_err(|error| Error::io(from, error))?;
        if read == 0 {
            break;
        }
        if block[..read].iter().all(|byte| *byte == 0) {
            target.seek(io::SeekFrom::Current(read as i64)).await.map_err(|error| Error::io(to, error))?;
        } else {
            target.write_all(&block[..read]).await.map_err(|error| Error::io(to, error))?;
        }
    }
    target.flush().await.map_err(|error| Error::io(to, error))?;
    target.set_len(length).await.map_err(|error| Error::io(to, error))?;
    target.sync_all().await.map_err(|error| Error::io(to, error))
}

/// Every file under `from` found under `to` at the same size: the
/// check a copy is trusted on. A file missing there, or of another
/// size, is [`Error::Size`].
fn same<'a>(from: &'a Path, to: &'a Path) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>> {
    Box::pin(async move {
        let meta = tokio::fs::symlink_metadata(from).await.map_err(|source| Error::io(from, source))?;
        if meta.is_symlink() {
            return Ok(());
        }
        if !meta.is_dir() {
            return match tokio::fs::symlink_metadata(to).await {
                Ok(copied) if copied.len() == meta.len() => Ok(()),
                _ => Err(Error::Size(to.to_path_buf())),
            };
        }
        let mut listed = tokio::fs::read_dir(from).await.map_err(|source| Error::io(from, source))?;
        loop {
            let entry = listed.next_entry().await.map_err(|source| Error::io(from, source))?;
            let Some(entry) = entry else {
                break;
            };
            same(&entry.path(), &to.join(entry.file_name())).await?;
        }
        Ok(())
    })
}

/// The bytes every file under `path` is long, together: what a copy
/// of it needs room for.
pub(super) fn size(path: &Path) -> Pin<Box<dyn Future<Output = Result<u64, Error>> + Send + '_>> {
    Box::pin(async move {
        let meta = tokio::fs::symlink_metadata(path).await.map_err(|source| Error::io(path, source))?;
        if !meta.is_dir() {
            return Ok(if meta.is_symlink() { 0 } else { meta.len() });
        }
        let mut total = 0;
        let mut listed = tokio::fs::read_dir(path).await.map_err(|source| Error::io(path, source))?;
        loop {
            let entry = listed.next_entry().await.map_err(|source| Error::io(path, source))?;
            let Some(entry) = entry else {
                break;
            };
            total += size(&entry.path()).await?;
        }
        Ok(total)
    })
}

/// Every folder under `path` that holds nothing removed, deepest
/// first, and `path` itself if that leaves it empty. What a transfer
/// leaves of the old folder once everything is carried out of it, and
/// of the new once an undo has taken everything back. Whether `path`
/// is empty now, and so gone.
pub(super) fn prune(path: &Path) -> Pin<Box<dyn Future<Output = Result<bool, Error>> + Send + '_>> {
    Box::pin(async move {
        let mut listed = match tokio::fs::read_dir(path).await {
            Ok(listed) => listed,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(true),
            Err(source) => return Err(Error::io(path, source)),
        };
        let mut empty = true;
        let mut folders: Vec<PathBuf> = Vec::new();
        loop {
            let entry = listed.next_entry().await.map_err(|source| Error::io(path, source))?;
            let Some(entry) = entry else {
                break;
            };
            if folder(&entry.path()).await? {
                folders.push(entry.path());
            } else {
                empty = false;
            }
        }
        drop(listed);
        for folder in folders {
            if !prune(&folder).await? {
                empty = false;
            }
        }
        if empty {
            tokio::fs::remove_dir(path).await.map_err(|source| Error::io(path, source))?;
        }
        Ok(empty)
    })
}
