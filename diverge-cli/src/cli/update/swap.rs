//! The installed programs replaced by the unpacked ones.

use std::io;
use std::path::Path;

use super::Error;

/// The interrupts a terminal sends, listened for and so not acted
/// on: while one of these is held, Ctrl-C — and on Unix a terminate —
/// does not end the process. Dropped, an interrupt ends it again as
/// it always did.
pub struct HeldOff {
    #[cfg(unix)]
    _signals: [Option<tokio::signal::unix::Signal>; 2],
    #[cfg(windows)]
    _signals: Option<tokio::signal::windows::CtrlC>,
}

/// Hold interrupts off from now. A host that will not let one be
/// listened for is not an error: the replacing is quick either way.
pub fn hold_off() -> HeldOff {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        HeldOff { _signals: [signal(SignalKind::interrupt()).ok(), signal(SignalKind::terminate()).ok()] }
    }
    #[cfg(windows)]
    {
        HeldOff { _signals: tokio::signal::windows::ctrl_c().ok() }
    }
    #[cfg(not(any(unix, windows)))]
    {
        HeldOff {}
    }
}

/// Each of `files`, in order, replaced in `bin` by the one of its
/// name in `new`, the one it replaces kept in `old`.
///
/// The rules:
///
/// 1. For each file: the installed one, where there is one, is
///    renamed from `bin` into `old`; then the new one is renamed from
///    `new` into `bin`. Both are renames within the one root, so
///    neither copies a byte, and a program that is running — this
///    one, on Windows — is renamed like any other.
/// 2. Interrupts are held off from the first rename to the last: see
///    [`hold_off`].
/// 3. A rename that fails ends the replacing, and every file already
///    replaced is put back, last first: the new one out of `bin`, the
///    old one into it. The error says whether all of them were.
/// 4. The caller orders `files` with the CLI last, so a replacing
///    that ends part way — the process killed outright — leaves the
///    CLI that began it.
pub async fn swap(bin: &Path, new: &Path, old: &Path, files: &[String]) -> Result<(), Error> {
    let _held = hold_off();
    let mut done: Vec<(&String, bool)> = Vec::with_capacity(files.len());
    for file in files {
        match one(bin, new, old, file).await {
            Ok(had) => done.push((file, had)),
            Err((source, had)) => {
                // What this file's own first rename did, undone, then
                // every file before it.
                let mut restored = !had || tokio::fs::rename(old.join(file), bin.join(file)).await.is_ok();
                for (file, had) in done.into_iter().rev() {
                    restored &= tokio::fs::rename(bin.join(file), new.join(file)).await.is_ok();
                    if had {
                        restored &= tokio::fs::rename(old.join(file), bin.join(file)).await.is_ok();
                    }
                }
                return Err(Error::Swap { file: file.clone(), source, restored });
            }
        }
    }
    Ok(())
}

/// One file replaced: whether there was an installed one, kept now in
/// `old`. A failure says whether the installed one had been moved
/// aside when it came.
async fn one(bin: &Path, new: &Path, old: &Path, file: &str) -> Result<bool, (io::Error, bool)> {
    let installed = bin.join(file);
    let had = tokio::fs::try_exists(&installed).await.map_err(|source| (source, false))?;
    if had {
        tokio::fs::rename(&installed, old.join(file)).await.map_err(|source| (source, false))?;
    }
    tokio::fs::rename(new.join(file), &installed).await.map_err(|source| (source, had))?;
    Ok(had)
}
