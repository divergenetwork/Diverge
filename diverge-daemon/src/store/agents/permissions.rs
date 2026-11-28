//! What an agent's container last declared it may ask of the daemon.

use diverge_sdk::shared::permission::Grant;
use sqlx::PgConnection;
use sqlx::types::Json;

use crate::store::{AgentId, Error};

/// Keep `permissions` as what the agent's container declared at its
/// registration when it last started: written once its run is up, and
/// empty for a run whose program declared none.
pub async fn set_permissions(conn: &mut PgConnection, id: AgentId, permissions: &[Grant]) -> Result<(), Error> {
    sqlx::query("UPDATE diverge.agents SET permissions = $2 WHERE id = $1")
        .bind(id.0)
        .bind(Json(permissions))
        .execute(&mut *conn)
        .await?;
    Ok(())
}
