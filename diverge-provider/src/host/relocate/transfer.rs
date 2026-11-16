//! The transfer: begun, undone, finished.

use std::io;
use std::path::{Path, PathBuf};

use super::moving::{self, MARKER, Mode};
use super::{Error, rewrite};

/// One thing a transfer did, kept so that it can be undone.
#[derive(Debug)]
pub enum Step {
    /// An entry renamed.
    Renamed {
        /// Where it was.
        from: PathBuf,
        /// Where it is.
        to: PathBuf,
    },
    /// An entry copied across filesystems; its source is there still.
    Copied {
        /// The source, deleted at the finish.
        from: PathBuf,
        /// The copy, removed by an undo.
        to: PathBuf,
    },
    /// A file's naming of the old path rewritten.
    Rewrote {
        /// The file.
        path: PathBuf,
        /// What it held.
        original: String,
    },
    /// On Windows: a WSL distribution's disk moved by WSL.
    Distribution {
        /// The distribution.
        name: String,
        /// The folder its disk was in.
        from: PathBuf,
        /// The folder it is in.
        to: PathBuf,
    },
}

/// Podman's data carried, and not yet let go of: what [`transfer`]
/// answers. The caller writes the new path to the configuration and
/// then calls [`finish`](Self::finish); if the write fails, it calls
/// [`undo`](Self::undo) instead. One that carried nothing does
/// nothing at either.
#[derive(Debug)]
pub struct Transfer {
    old: PathBuf,
    new: PathBuf,
    /// Whether this transfer made the new folder.
    made: bool,
    /// Whether anything was there to carry.
    carried: bool,
    /// Whether this took up a transfer left unfinished: rule 8.
    resumed: bool,
    steps: Vec<Step>,
}

/// Podman's data at `old` carried to `new`, both absolute: see
/// [`relocate`](super) for the rules. The provider is not running:
/// the caller holds its lock.
pub async fn transfer(old: &Path, new: &Path) -> Result<Transfer, Error> {
    #[cfg(not(target_os = "linux"))]
    let inside = |path: &Path, base: &Path| super::machine::under(path, base);
    #[cfg(target_os = "linux")]
    let inside = |path: &Path, base: &Path| path.starts_with(base);
    if inside(new, old) || inside(old, new) {
        return Err(Error::Nested { old: old.to_path_buf(), new: new.to_path_buf() });
    }
    let mut transfer = Transfer { old: old.to_path_buf(), new: new.to_path_buf(), made: false, carried: false, resumed: false, steps: Vec::new() };
    // Rule 8 first: a transfer left unfinished is taken up again
    // whatever is left at the old path, even nothing.
    let resumed = marked(old, new).await?;
    transfer.resumed = resumed;
    if !resumed {
        // Rule 1.
        if empty(old).await?.unwrap_or(true) {
            return Ok(transfer);
        }
        // Rule 3.
        match empty(new).await? {
            None => {
                tokio::fs::create_dir_all(new).await.map_err(|source| Error::io(new, source))?;
                transfer.made = true;
            }
            Some(true) => {}
            Some(false) => return Err(Error::Occupied(new.to_path_buf())),
        }
    }
    transfer.carried = true;
    // The old path as a file may name it: as given, and as the
    // filesystem resolves it, since podman may have written either.
    let mut olds = vec![rewrite::written(old)?];
    if let Ok(resolved) = tokio::fs::canonicalize(old).await {
        let resolved = rewrite::written(&resolved)?;
        if !olds.contains(&resolved) && !cfg!(windows) {
            olds.push(resolved);
        }
    }
    let marker = new.join(MARKER);
    let old_text = old.to_string_lossy().into_owned();
    tokio::fs::write(&marker, old_text.as_bytes()).await.map_err(|source| Error::io(&marker, source))?;
    match carried(&mut transfer, &olds, resumed).await {
        Ok(()) => Ok(transfer),
        Err(error) => {
            transfer.undo().await;
            Err(error)
        }
    }
}

/// The carrying itself, every step into `transfer.steps`: rules 4 to
/// 6 and 9.
async fn carried(transfer: &mut Transfer, olds: &[String], resumed: bool) -> Result<(), Error> {
    let (old, new) = (transfer.old.clone(), transfer.new.clone());
    let there = tokio::fs::metadata(&old).await.is_ok();
    if there {
        // Rule 4.
        #[cfg(target_os = "linux")]
        super::linux::fit(&old).await?;
        #[cfg(not(target_os = "linux"))]
        super::machine::still(&old).await?;
        // Rule 5, and rule 6's room.
        let mode = moving::mode(&old, &new).await?;
        if mode == Mode::Copy {
            let needed = moving::size(&old).await?;
            let free = super::space::free(&new).await?;
            if needed > free {
                return Err(Error::Space { path: new, needed, free });
            }
        }
        #[cfg(windows)]
        super::wsl::carry(&old, &new, &mut transfer.steps).await?;
        moving::carry(mode, &old, &new, resumed, &mut transfer.steps).await?;
    }
    #[cfg(not(target_os = "linux"))]
    {
        let new_text = rewrite::written(&new)?;
        let files = super::machine::references(&new).await?;
        for file in &files {
            let Ok(text) = tokio::fs::read_to_string(file).await else {
                continue;
            };
            let Some(rewritten) = rewrite::replace(&text, olds, &new_text) else {
                continue;
            };
            put(file, &rewritten).await?;
            transfer.steps.push(Step::Rewrote { path: file.clone(), original: text });
        }
        // Rule 9.
        for file in &files {
            if let Ok(text) = tokio::fs::read_to_string(file).await {
                if rewrite::names(&text, olds) {
                    return Err(Error::Remains(file.clone()));
                }
            }
        }
    }
    #[cfg(windows)]
    super::wsl::check(&transfer.steps).await?;
    let _ = olds;
    Ok(())
}

/// `text` put in the file at `path`, whole: written beside it and
/// renamed over it, so a reader finds the old file or the new.
async fn put(path: &Path, text: &str) -> Result<(), Error> {
    let mut beside = path.as_os_str().to_owned();
    beside.push(".diverge-tmp");
    let beside = PathBuf::from(beside);
    tokio::fs::write(&beside, text).await.map_err(|source| Error::io(&beside, source))?;
    tokio::fs::rename(&beside, path).await.map_err(|source| Error::io(path, source))
}

/// Whether the folder at `path` holds nothing; `None` where there is
/// no such folder.
async fn empty(path: &Path) -> Result<Option<bool>, Error> {
    let mut listed = match tokio::fs::read_dir(path).await {
        Ok(listed) => listed,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(Error::io(path, source)),
    };
    let first = listed.next_entry().await.map_err(|source| Error::io(path, source))?;
    Ok(Some(first.is_none()))
}

/// Whether `new` holds the marker of a transfer from `old` that did
/// not finish: rule 8.
async fn marked(old: &Path, new: &Path) -> Result<bool, Error> {
    match tokio::fs::read_to_string(new.join(MARKER)).await {
        Ok(from) => Ok(Path::new(from.trim()) == old),
        Err(_) => Ok(false),
    }
}

impl Transfer {
    /// Whether anything was carried: `false` where the old path held
    /// nothing.
    pub fn carried(&self) -> bool {
        self.carried
    }

    /// Everything put back where it was, the last step first: rule 7.
    /// A step that will not go back is passed over and the rest are
    /// still undone, since there is nothing better to do with it; the
    /// marker stays in the new folder then, so the same transfer asked
    /// for again takes up what is left. It stays, too, where this
    /// transfer took up one left unfinished: what that one carried is
    /// in no step here and is at the new path still, and only the
    /// marker says that the two folders are halves of one.
    pub async fn undo(mut self) {
        let mut whole = true;
        while let Some(step) = self.steps.pop() {
            let undone = match step {
                Step::Renamed { from, to } => tokio::fs::rename(&to, &from).await.is_ok(),
                Step::Copied { to, .. } => moving::remove(&to).await.is_ok(),
                Step::Rewrote { path, original } => put(&path, &original).await.is_ok(),
                #[cfg(windows)]
                Step::Distribution { name, from, .. } => super::wsl::moved(&name, &from).await.is_ok(),
                #[cfg(not(windows))]
                Step::Distribution { .. } => true,
            };
            whole &= undone;
        }
        if !whole || !self.carried || self.resumed {
            return;
        }
        let _ = tokio::fs::remove_file(self.new.join(MARKER)).await;
        if self.made {
            let _ = moving::prune(&self.new).await;
        }
    }

    /// The transfer let go of, once the new path is the
    /// configuration's: the source of every copy deleted — rule 6 —
    /// the marker removed, and what is left of the old folder, which
    /// is folders that hold nothing, removed with it. A source that
    /// will not be deleted is the error, with the data whole at the
    /// new path and the setting changed: what is left at the old path
    /// is the caller's to remove.
    pub async fn finish(self) -> Result<(), Error> {
        if !self.carried {
            return Ok(());
        }
        for step in &self.steps {
            if let Step::Copied { from, .. } = step {
                moving::remove(from).await?;
            }
        }
        let marker = self.new.join(MARKER);
        match tokio::fs::remove_file(&marker).await {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(source) => return Err(Error::io(&marker, source)),
        }
        moving::prune(&self.old).await?;
        Ok(())
    }
}
