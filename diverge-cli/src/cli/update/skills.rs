//! A release's skills: unpacked, and installed.

use std::io::{Cursor, Read as _};
use std::path::{Path, PathBuf};

use super::Error;

/// The skills zip at `archive` unpacked into `into`, its folders
/// kept, and the names of the skills it holds, in order.
///
/// The rules:
///
/// 1. A skill is one folder at the zip's top level, named for the
///    skill, holding its `SKILL.md` and whatever else it has. Its
///    files are written under `into` at the paths the zip gives them.
/// 2. A file at the top level, in no folder, is no part of any skill
///    and is passed over.
/// 3. An entry whose path would land outside `into` — one that climbs
///    with `..`, or is absolute — is refused, [`Error::Archive`], and
///    nothing more is written.
/// 4. A zip that holds no skill is [`Error::Archive`]: a release's
///    skills are never none.
///
/// The zip is read whole into memory, and each file as it is unpacked.
pub async fn unpack_skills(archive: &Path, into: &Path) -> Result<Vec<String>, Error> {
    let bytes = tokio::fs::read(archive).await.map_err(|source| Error::Io { path: archive.to_path_buf(), source })?;
    let broken = |error: &dyn std::fmt::Display| Error::Archive(error.to_string());
    let mut zip = zip::ZipArchive::new(Cursor::new(&bytes[..])).map_err(|error| broken(&error))?;
    let mut names: Vec<String> = Vec::new();
    for index in 0..zip.len() {
        let (path, data) = {
            let mut entry = zip.by_index(index).map_err(|error| broken(&error))?;
            if !entry.is_file() {
                continue;
            }
            // Rule 3.
            let Some(path) = entry.enclosed_name() else {
                return Err(Error::Archive("an entry would be written outside the skills".to_string()));
            };
            let mut data = Vec::new();
            entry.read_to_end(&mut data).map_err(|error| broken(&error))?;
            (path, data)
        };
        let mut parts = path.components();
        let (Some(skill), Some(_)) = (parts.next(), parts.next()) else {
            // Rule 2.
            continue;
        };
        let skill = skill.as_os_str().to_string_lossy().into_owned();
        if !names.contains(&skill) {
            names.push(skill);
        }
        let to = into.join(&path);
        if let Some(folder) = to.parent() {
            tokio::fs::create_dir_all(folder).await.map_err(|source| Error::Io { path: folder.to_path_buf(), source })?;
        }
        tokio::fs::write(&to, &data).await.map_err(|source| Error::Io { path: to.clone(), source })?;
    }
    // Rule 4.
    if names.is_empty() {
        return Err(Error::Archive("it holds no skill".to_string()));
    }
    Ok(names)
}

/// The skills `names`, unpacked under `from`, installed in `into`.
///
/// The rules:
///
/// 1. `into` is made if it is not there.
/// 2. Each skill replaces whatever `into` holds under its name: a
///    folder, or a file, of that name is removed whole first, so no
///    file of an older version of the skill is left beside the new.
/// 3. Nothing else in `into` is touched.
/// 4. A skill is copied, file by file, and not moved: `from` may be
///    on another drive than `into`.
pub async fn install_skills(from: &Path, names: &[String], into: &Path) -> Result<(), Error> {
    let io = |path: &Path| {
        let path = path.to_path_buf();
        move |source| Error::Io { path, source }
    };
    // Rule 1.
    tokio::fs::create_dir_all(into).await.map_err(io(into))?;
    for name in names {
        let to = into.join(name);
        // Rule 2.
        match tokio::fs::symlink_metadata(&to).await {
            Ok(there) if there.is_dir() => tokio::fs::remove_dir_all(&to).await.map_err(io(&to))?,
            Ok(_) => tokio::fs::remove_file(&to).await.map_err(io(&to))?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(source) => return Err(Error::Io { path: to, source }),
        }
        // Rule 4: every folder of the skill, the skill's own first.
        let mut folders: Vec<(PathBuf, PathBuf)> = vec![(from.join(name), to)];
        while let Some((source, target)) = folders.pop() {
            tokio::fs::create_dir_all(&target).await.map_err(io(&target))?;
            let mut listed = tokio::fs::read_dir(&source).await.map_err(io(&source))?;
            while let Some(entry) = listed.next_entry().await.map_err(io(&source))? {
                let (inner, placed) = (entry.path(), target.join(entry.file_name()));
                if entry.file_type().await.map_err(io(&inner))?.is_dir() {
                    folders.push((inner, placed));
                } else {
                    tokio::fs::copy(&inner, &placed).await.map_err(io(&placed))?;
                }
            }
        }
    }
    Ok(())
}
