//! The WSL distributions whose disks are under the old path, moved by
//! WSL.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use diverge_sdk::process;

use super::machine::under;
use super::{Error, Step};

/// The registry key WSL keeps its distributions under, in the
/// user's hive: one subkey per distribution, named by a GUID.
const LXSS: &str = r"Software\Microsoft\Windows\CurrentVersion\Lxss";

/// The disk's file name where a distribution's key names none.
const VHD: &str = "ext4.vhdx";

/// A WSL distribution, as the registry holds it.
#[derive(Debug, Clone)]
pub struct Distribution {
    /// Its name, `podman-diverge-provider` for the provider's machine.
    pub name: String,
    /// The folder its disk is in: `BasePath`.
    pub base: PathBuf,
    /// The disk's file name in that folder: `VhdFileName`.
    pub vhd: String,
}

/// Every WSL distribution of the user's, read from the registry. A
/// key with no name or no folder — one half made, or WSL's own
/// settings beside the distributions — is passed over.
pub async fn distributions() -> Result<Vec<Distribution>, Error> {
    let read = tokio::task::spawn_blocking(|| {
        let lxss = match windows_registry::CURRENT_USER.open(LXSS) {
            Ok(lxss) => lxss,
            // No key at all: WSL has never registered a distribution.
            Err(_) => return Ok(Vec::new()),
        };
        let mut found = Vec::new();
        for key in lxss.keys().map_err(|error| error.to_string())? {
            let Ok(distribution) = lxss.open(&key) else {
                continue;
            };
            let (Ok(name), Ok(base)) = (distribution.get_string("DistributionName"), distribution.get_string("BasePath")) else {
                continue;
            };
            let vhd = distribution.get_string("VhdFileName").unwrap_or_else(|_| VHD.to_string());
            let base = PathBuf::from(base.strip_prefix(r"\\?\").unwrap_or(&base));
            found.push(Distribution { name, base, vhd });
        }
        Ok::<_, String>(found)
    })
    .await;
    match read {
        Ok(Ok(found)) => Ok(found),
        Ok(Err(why)) => Err(Error::Registry(why)),
        Err(error) => Err(Error::Registry(error.to_string())),
    }
}

/// Every WSL distribution whose disk is under `old` moved to the same
/// place under `new`, by WSL itself, each recorded in `steps`.
///
/// A distribution's disk is WSL's: where it is, is in the registry,
/// and a disk renamed behind WSL is a distribution that no longer
/// starts. `wsl --manage <distribution> --move <folder>` is the one
/// way WSL offers that neither copies a disk that need only be
/// renamed nor forgets the distribution's settings: it makes the
/// folder, moves the disk into it — a rename on one volume, a copy
/// and a delete across two — and writes the folder to the registry,
/// putting the disk back if that fails. The rules:
///
/// - Every distribution under `old` is moved, not the provider's
///   alone: a machine another hand made with its data under the same
///   path has its disk in the folder that is moving.
/// - Each is terminated first; WSL moves no disk that is in use. One
///   that was not running is not an error.
/// - A distribution whose disk file is not there has nothing to
///   move, and is left as it is.
/// - One already registered under `new` — a transfer run again — is
///   not under `old`, and is not met here.
/// - A WSL that has no `--manage --move`, which is one older than
///   2.3.11, refuses the command, and its refusal is the error.
pub(super) async fn carry(old: &Path, new: &Path, steps: &mut Vec<Step>) -> Result<(), Error> {
    for distribution in distributions().await? {
        if !under(&distribution.base, old) {
            continue;
        }
        if tokio::fs::metadata(distribution.base.join(&distribution.vhd)).await.is_err() {
            continue;
        }
        let rest = relative(&distribution.base, old);
        let to = new.join(rest);
        let _ = wsl(&["--terminate", &distribution.name]).await;
        moved(&distribution.name, &to).await?;
        steps.push(Step::Distribution { name: distribution.name, from: distribution.base, to });
    }
    Ok(())
}

/// `base`, which is under `old`, less `old`: compared as
/// [`under`] compares, so a difference of case or of a `\\?\` does
/// not lose the rest.
fn relative(base: &Path, old: &Path) -> PathBuf {
    let depth = old.components().count();
    base.components().skip(depth).collect()
}

/// One distribution's disk moved to the folder `to`:
/// `wsl --manage <name> --move <to>`, required to succeed.
pub(super) async fn moved(name: &str, to: &Path) -> Result<(), Error> {
    let to = to.to_string_lossy().into_owned();
    match wsl(&["--manage", name, "--move", &to]).await {
        Ok(()) => Ok(()),
        Err(why) => Err(Error::Wsl {
            distribution: name.to_string(),
            why: format!("{why} (a WSL older than 2.3.11 cannot move one: `wsl --update`)"),
        }),
    }
}

/// Every distribution named in `steps` found registered under the
/// folder it was moved to: rule 9 of [`relocate`](super).
pub(super) async fn check(steps: &[Step]) -> Result<(), Error> {
    let registered = distributions().await?;
    for step in steps {
        let Step::Distribution { name, to, .. } = step else {
            continue;
        };
        let there = registered.iter().any(|distribution| distribution.name == *name && under(&distribution.base, to));
        if !there {
            return Err(Error::Wsl {
                distribution: name.clone(),
                why: format!("WSL does not hold it registered at `{}`", to.display()),
            });
        }
    }
    Ok(())
}

/// `wsl <args>`, waited for; what it said, where it refused. WSL
/// writes its refusals to stdout, and in UTF-16 unless it is told
/// otherwise, which `WSL_UTF8` tells it.
async fn wsl(args: &[&str]) -> Result<(), String> {
    let mut command = tokio::process::Command::new("wsl");
    command
        .args(args)
        .env("WSL_UTF8", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    process::no_window(&mut command);
    let output = command.output().await.map_err(|error| format!("wsl could not be started: {error}"))?;
    if output.status.success() {
        return Ok(());
    }
    let said = [output.stdout, output.stderr].concat();
    let said = String::from_utf8_lossy(&said).replace('\0', "");
    Err(format!("{}: {}", output.status, said.trim()))
}
