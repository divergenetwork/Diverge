//! Reading an image's answer off the record.

use diverge_sdk::shared::containers::info::Document;
use sqlx::{PgConnection, Row as _};
use sqlx::types::Json;

use crate::store::Error;

/// The document on record for the digest, or `None` when none is. A
/// row with no document, which only a database older than the rule
/// that only documents are recorded holds, is no record; and so is a
/// row whose document is none as a document is now read — one saved
/// before a document said its `type` — which is asked of its source
/// again.
pub async fn by_digest(conn: &mut PgConnection, digest: &str) -> Result<Option<Document>, Error> {
    let row = sqlx::query("SELECT document FROM diverge.image_info WHERE digest = $1")
        .bind(digest)
        .fetch_optional(&mut *conn)
        .await?;
    match row {
        Some(row) => {
            let document: Option<Json<serde_json::Value>> = row.try_get("document")?;
            Ok(document.and_then(|Json(document)| serde_json::from_value::<Document>(document).ok()))
        }
        None => Ok(None),
    }
}
