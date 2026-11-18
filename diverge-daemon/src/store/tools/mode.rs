//! How a tool on record is kept running, by its template.

use diverge_sdk::daemon::endpoints::standalone_tools::templates::Mode;
use sqlx::PgConnection;

use super::Tool;
use crate::store::{Error, tools_templates};

/// The mode the tool's template says: lazy, or an eager tool. `None`
/// for a tool whose template is gone.
pub async fn mode(conn: &mut PgConnection, tool: &Tool) -> Result<Option<Mode>, Error> {
    Ok(tools_templates::by_id(conn, &tool.template, false).await?.map(|record| record.template.r#type.mode))
}
