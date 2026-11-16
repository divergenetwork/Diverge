//! What must be so of podman's store on Linux before it is moved.

use std::path::{Path, PathBuf};

use super::Error;
use crate::host::tools::{capture, podman};

/// The store under `old` found fit to move: rule 4 of
/// [`relocate`](super).
///
/// - **Nothing is mounted at or under it.** A container's overlay, a
///   volume's scratch, or the path being a mount point of its own:
///   each is a rename the kernel refuses or a mount left pointing
///   nowhere. Read from `/proc/self/mountinfo`, whose fifth field is
///   the mount point. [`Error::Mounted`], naming them.
/// - **Its database holds no container, pod or volume.** Podman
///   records the graphroot in its database and compares it at every
///   start. With none of the three it rewrites the record and goes
///   on; with any it refuses to run, and a container's own record
///   holds absolute paths besides. A provider that stopped removed
///   its containers, so what is found here another hand made.
///   [`Error::Held`], counting them. Asked of podman itself, told
///   that `old` is its root; and only where the store has a
///   database, since a store never used has nothing in it and podman
///   need not be installed to move an empty folder.
pub(super) async fn fit(old: &Path) -> Result<(), Error> {
    let mounts = mounted(old).await?;
    if !mounts.is_empty() {
        return Err(Error::Mounted(mounts));
    }
    if tokio::fs::metadata(old.join("db.sql")).await.is_err() {
        return Ok(());
    }
    podman::configure(old.to_path_buf());
    let containers = count(["ps", "--all", "--quiet"]).await?;
    let pods = count(["pod", "ls", "--quiet"]).await?;
    let volumes = count(["volume", "ls", "--quiet"]).await?;
    if containers + pods + volumes > 0 {
        return Err(Error::Held { containers, pods, volumes });
    }
    Ok(())
}

/// How many lines `podman <args>` answers: one per thing listed.
async fn count<const N: usize>(args: [&str; N]) -> Result<usize, Error> {
    let answer = capture("podman", podman::command(args)).await?;
    Ok(answer.lines().filter(|line| !line.trim().is_empty()).count())
}

/// Every mount point at or under `old`.
async fn mounted(old: &Path) -> Result<Vec<PathBuf>, Error> {
    let table = Path::new("/proc/self/mountinfo");
    let text = tokio::fs::read_to_string(table).await.map_err(|source| Error::io(table, source))?;
    Ok(text
        .lines()
        .filter_map(|line| line.split(' ').nth(4))
        .map(|point| PathBuf::from(point.replace("\\040", " ")))
        .filter(|point| point.starts_with(old))
        .collect())
}
