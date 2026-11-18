//! The log folders that are there, less those no record owns.

use std::collections::HashSet;
use std::path::Path;

use sqlx::PgConnection;

use super::Error;

/// The books whose folders are named by a row's id: the folder under
/// the daemon's directory, and the table the ids are of. The
/// administrators' book is not among them: its folders are named by
/// an identity's key, no table holds them, and none is ever removed.
pub const BOOKS: [(&str, &str); 5] = [
    ("agents", "agents"),
    ("tools", "tools"),
    ("dependency_tools", "dependency_tools"),
    ("connected_standalone_tools", "connected_standalone_tools"),
    ("accounts", "accounts"),
];

/// Every log folder of the book `folder` under `logs` that a row of
/// `table` owns, by the row's id — every other removed first, whole:
///
/// - a folder named by a decimal id no row of the table has is one
///   whose record was deleted, the delete's transaction committed and
///   the daemon ended before the folder went. It is removed;
/// - a name that is no decimal id is not a log of the book, and is
///   left;
/// - a book's folder that is not there holds no log.
pub async fn owned(conn: &mut PgConnection, logs: &Path, folder: &str, table: &str) -> Result<Vec<i64>, Error> {
    let ids: Vec<i64> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT id FROM diverge.{table}")))
        .fetch_all(&mut *conn)
        .await
        .map_err(crate::store::Error::from)?;
    let ids: HashSet<i64> = ids.into_iter().collect();
    let dir = logs.join(folder);
    let mut listed = match tokio::fs::read_dir(&dir).await {
        Ok(listed) => listed,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(Error::Folder { path: dir, source }),
    };
    let mut owned = Vec::new();
    loop {
        let entry = match listed.next_entry().await {
            Ok(Some(entry)) => entry,
            Ok(None) => break,
            Err(source) => return Err(Error::Folder { path: dir, source }),
        };
        let Some(id) = entry.file_name().to_str().and_then(|name| name.parse::<i64>().ok()) else {
            continue;
        };
        if ids.contains(&id) {
            owned.push(id);
            continue;
        }
        let path = entry.path();
        match tokio::fs::remove_dir_all(&path).await {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(source) => return Err(Error::Folder { path, source }),
        }
    }
    Ok(owned)
}

/// Every folder of the administrators' book under `logs`: each one a
/// command log, none removed.
pub async fn admins(logs: &Path) -> Result<Vec<std::path::PathBuf>, Error> {
    let dir = logs.join("admins");
    let mut listed = match tokio::fs::read_dir(&dir).await {
        Ok(listed) => listed,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(Error::Folder { path: dir, source }),
    };
    let mut folders = Vec::new();
    loop {
        match listed.next_entry().await {
            Ok(Some(entry)) => folders.push(entry.path()),
            Ok(None) => break,
            Err(source) => return Err(Error::Folder { path: dir, source }),
        }
    }
    Ok(folders)
}
