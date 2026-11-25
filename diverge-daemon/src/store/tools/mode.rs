//! How a tool on record is kept running, by its template.

use std::collections::HashMap;

use diverge_sdk::shared::containers::info::Mode;
use sqlx::{PgConnection, Row as _};

use super::Tool;
use crate::store::Error;

/// Every tool template's id, joined to the `mode` its image's
/// document says. A template's image is named by its digest, and the
/// document of an image is on record by that digest: a tool is made
/// only of an image whose document was read.
const SELECT: &str = "SELECT t.id, i.document->>'mode' AS mode \
     FROM diverge.tools_templates t JOIN diverge.image_info i ON i.digest = t.template->'image'->>'digest'";

/// The mode a word of the document names, or `None` for a word that
/// is neither, and for a document that says none: an agent's.
fn named(word: Option<String>) -> Option<Mode> {
    match word.as_deref() {
        Some("lazy") => Some(Mode::Lazy),
        Some("eager") => Some(Mode::Eager),
        _ => None,
    }
}

/// The mode the tool's image says: lazy, or an eager tool. `None`
/// for a tool whose template is gone, or whose image's document is
/// not on record.
pub async fn mode(conn: &mut PgConnection, tool: &Tool) -> Result<Option<Mode>, Error> {
    let row = sqlx::query(sqlx::AssertSqlSafe(format!("{SELECT} WHERE t.id = $1")))
        .bind(&tool.template)
        .fetch_optional(&mut *conn)
        .await?;
    Ok(match row {
        Some(row) => named(row.try_get("mode")?),
        None => None,
    })
}

/// The mode of every tool template's image, by the template's id: a
/// template whose image says none is not among them.
pub async fn modes(conn: &mut PgConnection) -> Result<HashMap<String, Mode>, Error> {
    let rows = sqlx::query(SELECT).fetch_all(&mut *conn).await?;
    let mut modes = HashMap::with_capacity(rows.len());
    for row in rows {
        if let Some(mode) = named(row.try_get("mode")?) {
            modes.insert(row.try_get("id")?, mode);
        }
    }
    Ok(modes)
}
