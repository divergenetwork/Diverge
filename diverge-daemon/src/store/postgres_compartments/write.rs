//! Writing compartments.

use diverge_sdk::daemon::endpoints::postgres::compartments::{Compartment, Kind};
use sqlx::PgConnection;
use sqlx::types::Json;

use super::kind_name;
use crate::database;
use crate::store::Error;

/// Put the compartment on record under its role, holding no tag yet.
/// One on record already is left as it is, tags and all.
pub async fn register(conn: &mut PgConnection, compartment: &Compartment, kind: Kind) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO diverge.postgres_compartments (role, compartment, kind) VALUES ($1, $2, $3) ON CONFLICT (role) DO NOTHING",
    )
    .bind(database::role_of(compartment))
    .bind(Json(compartment))
    .bind(kind_name(kind))
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Make the compartment's tags exactly `tags`, which the caller has
/// sorted and deduplicated with [`tags::with`](crate::store::tags::with)
/// or [`tags::without`](crate::store::tags::without).
pub async fn set_tags(conn: &mut PgConnection, role: &str, tags: &[String]) -> Result<(), Error> {
    sqlx::query("UPDATE diverge.postgres_compartments SET tags = $2 WHERE role = $1")
        .bind(role)
        .bind(tags)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Take the compartment off record. Its role and its schema are the
/// caller's to drop, in the same transaction.
pub async fn delete(conn: &mut PgConnection, role: &str) -> Result<(), Error> {
    sqlx::query("DELETE FROM diverge.postgres_compartments WHERE role = $1")
        .bind(role)
        .execute(&mut *conn)
        .await?;
    Ok(())
}
