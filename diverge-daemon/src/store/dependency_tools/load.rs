//! Loading dependency tools.

use diverge_sdk::daemon::reference;
use diverge_sdk::shared::containers::info::Scope;
use sqlx::PgConnection;

use super::row::{self, ORDER, SELECT};
use super::{DependencyTool, scope_name};
use crate::store::{AgentId, DependencyToolId, Error, agents};

/// Every dependency tool, oldest first.
pub async fn all(conn: &mut PgConnection) -> Result<Vec<DependencyTool>, Error> {
    let rows = sqlx::query(sqlx::AssertSqlSafe(format!("{SELECT} {ORDER}"))).fetch_all(&mut *conn).await?;
    rows.iter().map(row::tool).collect()
}

/// The tool with the id, if any. With `lock`, its row is locked for
/// the transaction.
pub async fn by_id(conn: &mut PgConnection, id: DependencyToolId, lock: bool) -> Result<Option<DependencyTool>, Error> {
    let locking = if lock { " FOR UPDATE OF t" } else { "" };
    let row = sqlx::query(sqlx::AssertSqlSafe(format!("{SELECT} WHERE t.id = $1{locking}")))
        .bind(id.0)
        .fetch_optional(&mut *conn)
        .await?;
    row.as_ref().map(row::tool).transpose()
}

/// The tool under the scope key, if any: the template, the scope, and
/// the owner the scope names — the agent, or nobody. With `lock`, its
/// row is locked for the transaction.
pub async fn by_scope(
    conn: &mut PgConnection,
    template: &str,
    scope: Scope,
    owner_agent: Option<AgentId>,
    lock: bool,
) -> Result<Option<DependencyTool>, Error> {
    let locking = if lock { " FOR UPDATE OF t" } else { "" };
    let row = sqlx::query(sqlx::AssertSqlSafe(format!(
        "{SELECT} WHERE t.template = $1 AND t.scope = $2 AND t.owner_agent IS NOT DISTINCT FROM $3{locking}"
    )))
    .bind(template)
    .bind(scope_name(scope))
    .bind(owner_agent.map(|agent| agent.0))
    .fetch_optional(&mut *conn)
    .await?;
    row.as_ref().map(row::tool).transpose()
}

/// The tool the reference names, if any: its template under its
/// owner — an agent, named either way and found or not, or nobody.
/// With `lock`, its row is locked for the transaction.
pub async fn by_reference(conn: &mut PgConnection, reference: &reference::Dependency, lock: bool) -> Result<Option<DependencyTool>, Error> {
    let template = reference.template.as_str();
    match &reference.owner {
        reference::Owner::Agent(agent) => {
            let Some(agent) = agents::by_reference(conn, agent, false).await? else {
                return Ok(None);
            };
            by_scope(conn, template, Scope::Instance, Some(agent.id), lock).await
        }
        reference::Owner::Global => by_scope(conn, template, Scope::Global, None, lock).await,
    }
}

/// Every tool of the template, under every owner. With `lock`, each
/// row is locked for the transaction.
pub async fn of_template(conn: &mut PgConnection, template: &str, lock: bool) -> Result<Vec<DependencyTool>, Error> {
    let locking = if lock { " FOR UPDATE OF t" } else { "" };
    let rows = sqlx::query(sqlx::AssertSqlSafe(format!("{SELECT} WHERE t.template = $1 {ORDER}{locking}")))
        .bind(template)
        .fetch_all(&mut *conn)
        .await?;
    rows.iter().map(row::tool).collect()
}

/// Every tool owned by the agent: the instance ones of its own.
pub async fn of_owner_agent(conn: &mut PgConnection, agent: AgentId) -> Result<Vec<DependencyTool>, Error> {
    let rows = sqlx::query(sqlx::AssertSqlSafe(format!("{SELECT} WHERE t.owner_agent = $1 {ORDER}")))
        .bind(agent.0)
        .fetch_all(&mut *conn)
        .await?;
    rows.iter().map(row::tool).collect()
}

/// Every tool the agent is attached to, in the tools' order.
pub async fn of_agent(conn: &mut PgConnection, agent: AgentId) -> Result<Vec<DependencyTool>, Error> {
    let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
        "{SELECT} WHERE t.id IN (SELECT tool FROM diverge.dependency_attachments WHERE agent = $1) {ORDER}"
    )))
    .bind(agent.0)
    .fetch_all(&mut *conn)
    .await?;
    rows.iter().map(row::tool).collect()
}
