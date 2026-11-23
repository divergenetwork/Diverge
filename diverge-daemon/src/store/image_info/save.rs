//! Putting an image's answer on record.

use diverge_sdk::shared::containers::info::Document;
use sqlx::PgConnection;
use sqlx::types::Json;

use crate::store::Error;

/// Record that the image under the digest ships `document`. One on
/// record already is replaced by it. The same digest is the same image
/// and so the same file, but what a document may be has changed
/// before, and a record written under an older rule is one
/// [`by_digest`](super::by_digest) no longer reads: left as it was, it
/// would be asked for again on every use and never be on record.
pub async fn save(conn: &mut PgConnection, digest: &str, document: &Document) -> Result<(), Error> {
    sqlx::query("INSERT INTO diverge.image_info (digest, document) VALUES ($1, $2) ON CONFLICT (digest) DO UPDATE SET document = EXCLUDED.document")
        .bind(digest)
        .bind(Json(document))
        .execute(&mut *conn)
        .await?;
    Ok(())
}
