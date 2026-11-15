//! Putting an image's icon on record.

use sqlx::PgConnection;

use super::Which;
use crate::store::Error;

/// Record that the image under the digest ships `bytes` as the icon
/// `which`. One on record already is left as it is, since the same
/// digest is the same image and so the same icon.
pub async fn save(conn: &mut PgConnection, digest: &str, which: Which, bytes: &[u8]) -> Result<(), Error> {
    sqlx::query("INSERT INTO diverge.image_icons (digest, icon, bytes) VALUES ($1, $2, $3) ON CONFLICT (digest, icon) DO NOTHING")
        .bind(digest)
        .bind(which.member())
        .bind(bytes)
        .execute(&mut *conn)
        .await?;
    Ok(())
}
