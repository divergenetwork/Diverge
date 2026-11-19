//! What the store keeps of an administrator: its tags.
//!
//! An administrator is an identity the daemon's configuration names
//! that is no account's. It is an account in every respect but that it
//! is not stored: no row of `accounts` is it. Its tags are the one
//! thing of it a request changes, and they are kept here, by its
//! identity.

use std::collections::HashMap;

use diverge_sdk::shared::Identity;
use sqlx::{PgConnection, Row};

use crate::store::Error;

/// The tags of the administrator `identity`: none where none were ever
/// set. With `lock`, its row is made if it is not there and locked for
/// the transaction, so that a change of the tags reads and writes them
/// with no other between.
pub async fn tags(conn: &mut PgConnection, identity: &Identity, lock: bool) -> Result<Vec<String>, Error> {
    if lock {
        sqlx::query("INSERT INTO diverge.admin_tags (issuer, subject) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(&identity.issuer)
            .bind(&identity.subject)
            .execute(&mut *conn)
            .await?;
    }
    let locking = if lock { " FOR UPDATE" } else { "" };
    let row = sqlx::query(sqlx::AssertSqlSafe(format!("SELECT tags FROM diverge.admin_tags WHERE issuer = $1 AND subject = $2{locking}")))
        .bind(&identity.issuer)
        .bind(&identity.subject)
        .fetch_optional(&mut *conn)
        .await?;
    Ok(match row {
        Some(row) => row.try_get("tags")?,
        None => Vec::new(),
    })
}

/// Every identity with tags kept here, and its tags: what a listing of
/// the administrators reads once.
pub async fn all(conn: &mut PgConnection) -> Result<HashMap<Identity, Vec<String>>, Error> {
    let rows = sqlx::query("SELECT issuer, subject, tags FROM diverge.admin_tags").fetch_all(&mut *conn).await?;
    let mut all = HashMap::new();
    for row in rows {
        let identity = Identity {
            issuer: row.try_get("issuer")?,
            subject: row.try_get("subject")?,
        };
        all.insert(identity, row.try_get("tags")?);
    }
    Ok(all)
}

/// Make the administrator's tags exactly `tags`, which the caller has
/// sorted and deduplicated with [`tags::with`](crate::store::tags::with)
/// or [`tags::without`](crate::store::tags::without).
pub async fn set_tags(conn: &mut PgConnection, identity: &Identity, tags: &[String]) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO diverge.admin_tags (issuer, subject, tags) VALUES ($1, $2, $3) \
         ON CONFLICT (issuer, subject) DO UPDATE SET tags = EXCLUDED.tags",
    )
    .bind(&identity.issuer)
    .bind(&identity.subject)
    .bind(tags)
    .execute(&mut *conn)
    .await?;
    Ok(())
}
