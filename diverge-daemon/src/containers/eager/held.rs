//! An eager tool's attachments, read anew.

use std::sync::Arc;

use diverge_sdk::daemon::lifecycle::Cause;
use diverge_sdk::shared::containers::info::Mode;
use sqlx::PgConnection;

use super::{keep, unkeep};
use crate::containers::ToolKey;
use crate::daemon::Daemon;
use crate::store::{self, ToolId, dependency_tools, tools};

/// What may hold the tool as an eager tool changed: whether any agent
/// is attached to it is read anew from the records, and the tool kept
/// or kept no more by it. The rules:
///
/// 1. **Eager, and attached.** A tool on record whose image's mode is
///    `eager`, attached to an agent or to an agent template; a
///    dependency tool whose image's mode is, declared by any agent's
///    latest registration. Its [`Holds`](crate::containers::Holds) say
///    so — rule 8 of [`idle_tool`](crate::containers::idle_tool) — and
///    it is kept: its supervisor started if it had none, its first
///    start deployed by `cause`.
/// 2. **Anything else** — a lazy tool, an eager tool with nothing
///    attached, a connected tool, a tool whose row is gone — is not
///    held by attachment and is kept no more: its supervisor is
///    stilled, and its run, if one is up, is left to its idle clock,
///    which counts from here.
/// 3. **A store that cannot be read says nothing.** The tool is left
///    as it was, rather than an eager tool let go for want of an
///    answer.
pub async fn held(daemon: &Arc<Daemon>, key: ToolKey, cause: Cause) {
    let attached = match daemon.store.acquire().await {
        Ok(mut conn) => attached(&mut conn, key).await,
        Err(error) => Err(error),
    };
    // Rule 3.
    let Ok(attached) = attached else {
        return;
    };
    daemon.live.holds(key).await.attached.send_replace(attached);
    if attached {
        // Rule 1.
        keep(daemon, key, cause).await;
    } else {
        // Rule 2.
        unkeep(daemon, key).await;
        daemon.live.release_holds(key).await;
    }
}

/// Whether the tool is an eager tool with an agent attached, from the
/// records.
async fn attached(conn: &mut PgConnection, key: ToolKey) -> Result<bool, store::Error> {
    match key {
        ToolKey::Record(id) => {
            let Some(tool) = tools::by_id(conn, id, false).await? else {
                return Ok(false);
            };
            if tools::mode(conn, &tool).await? != Some(Mode::Eager) {
                return Ok(false);
            }
            attached_anywhere(conn, id).await
        }
        ToolKey::Dependency(id) => {
            let Some(tool) = dependency_tools::by_id(conn, id, false).await? else {
                return Ok(false);
            };
            if !tool.is_eager() {
                return Ok(false);
            }
            Ok(!dependency_tools::attachments::attached(conn, id).await?.is_empty())
        }
        // A connected tool is never an eager tool.
        ToolKey::Connected(_) => Ok(false),
    }
}

/// Whether the tool on record is attached to any agent or any agent
/// template, from the records.
async fn attached_anywhere(conn: &mut PgConnection, id: ToolId) -> Result<bool, store::Error> {
    if !tools::attachments::of_tool(conn, id).await?.is_empty() {
        return Ok(true);
    }
    Ok(!tools::template_attachments::of_tool(conn, id).await?.is_empty())
}
