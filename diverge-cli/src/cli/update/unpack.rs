//! An archive, unpacked.

use std::io::{Cursor, Read as _};
use std::path::Path;

use super::Error;

/// The programs every release holds, by name without a suffix. The
/// first five take the operating system's executable suffix; the
/// container proxy runs inside Linux containers whatever the host,
/// and never has one.
pub const PROGRAMS: [&str; 5] = ["diverge", "diverge-daemon", "diverge-provider", "diverge-postgres", "diverge-viewer"];

/// The container proxy's file name, on every operating system.
pub const PROXY: &str = "diverge-container-proxy";

/// What an executable's name ends in on `os`: `.exe` on Windows,
/// nothing elsewhere.
pub fn suffix(os: &str) -> &'static str {
    if os == "windows" { ".exe" } else { "" }
}

/// Every file a release for `os` must hold, the CLI last: the order
/// they are replaced in.
pub fn expected(os: &str) -> Vec<String> {
    let mut files: Vec<String> = PROGRAMS[1..].iter().map(|program| format!("{program}{}", suffix(os))).collect();
    files.push(PROXY.to_string());
    files.push(format!("{}{}", PROGRAMS[0], suffix(os)));
    files
}

/// Every file of the archive at `archive` written into `into`, flat,
/// and their names.
///
/// The rules:
///
/// 1. The archive is a zip where its name ends in `.zip`, and a
///    gzipped tar otherwise, as a release packs them.
/// 2. Only files are taken. Each is written under its own file name
///    and nothing of the path before it: a release holds one folder
///    of programs, and a name can lead nowhere outside `into`.
/// 3. On Unix a file is given the mode the archive says, so a program
///    is executable.
/// 4. An archive that cannot be read to its end — a download cut
///    short among them — is [`Error::Archive`].
///
/// The archive is read whole into memory, and each file as it is
/// unpacked.
pub async fn unpack(archive: &Path, into: &Path) -> Result<Vec<String>, Error> {
    let bytes = tokio::fs::read(archive).await.map_err(|source| Error::Io { path: archive.to_path_buf(), source })?;
    tokio::fs::create_dir_all(into).await.map_err(|source| Error::Io { path: into.to_path_buf(), source })?;
    let broken = |error: &dyn std::fmt::Display| Error::Archive(error.to_string());
    let mut names = Vec::new();
    if archive.extension().is_some_and(|extension| extension == "zip") {
        let mut zip = zip::ZipArchive::new(Cursor::new(&bytes[..])).map_err(|error| broken(&error))?;
        for index in 0..zip.len() {
            let (name, mode, data) = {
                let mut entry = zip.by_index(index).map_err(|error| broken(&error))?;
                if !entry.is_file() {
                    continue;
                }
                let name = entry.enclosed_name().and_then(|path| path.file_name().map(|name| name.to_string_lossy().into_owned()));
                let mode = entry.unix_mode();
                let mut data = Vec::new();
                entry.read_to_end(&mut data).map_err(|error| broken(&error))?;
                (name, mode, data)
            };
            if let Some(name) = name {
                write(into, &name, &data, mode).await?;
                names.push(name);
            }
        }
    } else {
        let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(&bytes[..]));
        for entry in tar.entries().map_err(|error| broken(&error))? {
            let mut entry = entry.map_err(|error| broken(&error))?;
            if !entry.header().entry_type().is_file() {
                continue;
            }
            let name = entry.path().map_err(|error| broken(&error))?.file_name().map(|name| name.to_string_lossy().into_owned());
            let mode = entry.header().mode().ok();
            let mut data = Vec::new();
            entry.read_to_end(&mut data).map_err(|error| broken(&error))?;
            if let Some(name) = name {
                write(into, &name, &data, mode).await?;
                names.push(name);
            }
        }
    }
    Ok(names)
}

/// One unpacked file written, with its mode where the host has one.
async fn write(into: &Path, name: &str, data: &[u8], mode: Option<u32>) -> Result<(), Error> {
    let path = into.join(name);
    let failed = |source| Error::Io { path: path.clone(), source };
    tokio::fs::write(&path, data).await.map_err(failed)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let permissions = std::fs::Permissions::from_mode(mode.unwrap_or(0o755));
        tokio::fs::set_permissions(&path, permissions).await.map_err(failed)?;
    }
    #[cfg(not(unix))]
    let _ = mode;
    Ok(())
}
