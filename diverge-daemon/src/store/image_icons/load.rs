//! Reading an image's icon off the record.

use sqlx::{PgConnection, Row as _};

use super::Which;
use crate::store::Error;

/// The bytes on record for the digest's icon, or `None` when none is.
pub async fn by_digest(conn: &mut PgConnection, digest: &str, which: Which) -> Result<Option<Vec<u8>>, Error> {
    let row = sqlx::query("SELECT bytes FROM diverge.image_icons WHERE digest = $1 AND icon = $2")
        .bind(digest)
        .bind(which.member())
        .fetch_optional(&mut *conn)
        .await?;
    match row {
        Some(row) => Ok(Some(row.try_get("bytes")?)),
        None => Ok(None),
    }
}
