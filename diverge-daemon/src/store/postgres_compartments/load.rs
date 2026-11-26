//! Loading compartments.

use chrono::{DateTime, Utc};
use diverge_sdk::daemon::endpoints::postgres::compartments::Compartment;
use sqlx::postgres::PgRow;
use sqlx::types::Json;
use sqlx::{PgConnection, Row as _};

use super::{Entry, kind_of};
use crate::store::Error;

/// The columns every compartment query selects.
const SELECT: &str = "SELECT role, compartment, kind, tags, created FROM diverge.postgres_compartments";

/// The compartment a row of [`SELECT`] holds.
fn entry(row: &PgRow) -> Result<Entry, Error> {
    let Json(compartment): Json<Compartment> = row.try_get("compartment")?;
    let kind: String = row.try_get("kind")?;
    let Some(kind) = kind_of(&kind) else {
        return Err(Error::Json(serde::de::Error::custom(format!("`{kind}` is not a compartment's kind"))));
    };
    let created: DateTime<Utc> = row.try_get("created")?;
    Ok(Entry {
        role: row.try_get("role")?,
        compartment,
        kind,
        tags: row.try_get("tags")?,
        created,
    })
}

/// Every compartment, oldest first.
pub async fn all(conn: &mut PgConnection) -> Result<Vec<Entry>, Error> {
    let rows = sqlx::query(sqlx::AssertSqlSafe(format!("{SELECT} ORDER BY created, role")))
        .fetch_all(&mut *conn)
        .await?;
    rows.iter().map(entry).collect()
}

/// The compartment of the role, if it is on record. With `lock`, its
/// row is locked for the transaction.
pub async fn by_role(conn: &mut PgConnection, role: &str, lock: bool) -> Result<Option<Entry>, Error> {
    let locking = if lock { " FOR UPDATE" } else { "" };
    let row = sqlx::query(sqlx::AssertSqlSafe(format!("{SELECT} WHERE role = $1{locking}")))
        .bind(role)
        .fetch_optional(&mut *conn)
        .await?;
    row.as_ref().map(entry).transpose()
}
