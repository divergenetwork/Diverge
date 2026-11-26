//! Removing dependency tools.

use sqlx::PgConnection;

use super::{DependencyTool, of_owner_agent, of_template};
use crate::store::{AgentId, Error};

/// Delete every tool of the template, under every owner; their
/// attachments go with them. The rows removed, for their states to be
/// retired; their compartments, and that no run is up, are the
/// caller's.
pub async fn delete_of_template(conn: &mut PgConnection, template: &str) -> Result<Vec<DependencyTool>, Error> {
    let removed = of_template(conn, template, true).await?;
    sqlx::query("DELETE FROM diverge.dependency_tools WHERE template = $1")
        .bind(template)
        .execute(&mut *conn)
        .await?;
    Ok(removed)
}

/// Delete every tool the agent owns: what the agent's delete does.
/// The rows removed, likewise.
pub async fn delete_of_owner_agent(conn: &mut PgConnection, agent: AgentId) -> Result<Vec<DependencyTool>, Error> {
    let removed = of_owner_agent(conn, agent).await?;
    sqlx::query("DELETE FROM diverge.dependency_tools WHERE owner_agent = $1")
        .bind(agent.0)
        .execute(&mut *conn)
        .await?;
    Ok(removed)
}
