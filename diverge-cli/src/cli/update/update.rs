//! One update.

use std::path::Path;

use super::{Error, download, expected, file, next, swap, unpack};
use crate::arguments;
use crate::cli::setting::hold;
use crate::lifecycle::{self, Program};

/// The folder beneath the root the installed programs are in.
pub const BIN: &str = "bin";

/// The folder beneath the root an update works in, one folder per
/// version inside it.
pub const TEMP: &str = "temp";

/// The programs under `root` replaced by another version's, as the
/// [module](super) states it, step by step:
///
/// 1. The platform: what was asked for, else this host's.
/// 2. The version: what was asked for, else one version up — see
///    [`next`] — and where there is none, one line saying so and
///    nothing else.
/// 3. The archive's name, from the release's own listing: a version
///    there is none of, or one not built for the platform, ends here.
/// 4. The three programs' locks, taken and held to the end.
/// 5. `temp/<version>` deleted if it is there, and made.
/// 6. The archive downloaded into it, and unpacked into `new` inside
///    it.
/// 7. Every program a release holds found among what was unpacked.
/// 8. The programs replaced, the CLI last: see [`swap`].
/// 9. `temp/<version>` deleted, as far as it can be: on Windows the
///    CLI that is running is in it and stays until the next update of
///    that version deletes the folder. One line says what was
///    installed.
pub async fn update(root: &Path, asked: arguments::Update) -> Result<(), crate::cli::Error> {
    let os = asked.os.unwrap_or_else(|| std::env::consts::OS.to_string());
    let arch = asked.arch.unwrap_or_else(|| std::env::consts::ARCH.to_string());
    let current = env!("CARGO_PKG_VERSION");
    let address = super::address();
    let client = super::client()?;
    let version = match asked.version {
        Some(version) => version,
        None => match next(&client, &address, current).await? {
            Some(version) => version,
            None => return Ok(lifecycle::say(&format!("{current} is the newest version")).await?),
        },
    };
    let archive = file(&client, &address, &version, &os, &arch).await?;
    // Held to the end of the function: no program starts under it.
    let _daemon = hold(root, Program::Daemon).await?;
    let _provider = hold(root, Program::Provider).await?;
    let _viewer = hold(root, Program::Viewer).await?;
    let work = root.join(TEMP).join(&version);
    let io = |path: &Path| {
        let path = path.to_path_buf();
        move |source| Error::Io { path, source }
    };
    match tokio::fs::remove_dir_all(&work).await {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => return Err(Error::Io { path: work, source }.into()),
    }
    let (new, old, bin) = (work.join("new"), work.join("old"), root.join(BIN));
    for folder in [&new, &old, &bin] {
        tokio::fs::create_dir_all(folder).await.map_err(io(folder))?;
    }
    let downloaded = work.join(&archive);
    download(&client, &address, &version, &os, &arch, &downloaded).await?;
    let unpacked = unpack(&downloaded, &new).await?;
    let files = expected(&os);
    if let Some(missing) = files.iter().find(|file| !unpacked.contains(file)) {
        return Err(Error::Incomplete(missing.clone()).into());
    }
    swap(&bin, &new, &old, &files).await?;
    let _ = tokio::fs::remove_dir_all(&work).await;
    Ok(lifecycle::say(&format!("updated to {version} for {os}/{arch}")).await?)
}
