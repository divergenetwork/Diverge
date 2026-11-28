//! Every eager tool brought up at the daemon's start.

use std::sync::Arc;

use diverge_sdk::daemon::lifecycle::Cause;
use diverge_sdk::shared::containers::info::Mode;

use super::held;
use crate::containers::ToolKey;
use crate::daemon::Daemon;
use crate::store::{self, dependency_tools, tools};

/// At the daemon's start: every eager tool on record with an agent or an
/// agent template attached is kept — its supervisor starts its
/// container, as a restart — a
/// dependency tool whose template is an eager tool, and a tool on record
/// whose image's mode is. One with none attached is left as it is:
/// nothing holds it, and the next attach keeps it up.
pub async fn boot(daemon: &Arc<Daemon>) -> Result<(), store::Error> {
    let kept = {
        let mut conn = daemon.store.acquire().await?;
        let mut kept = Vec::new();
        for tool in dependency_tools::all(&mut conn).await? {
            if tool.is_eager() && !dependency_tools::attachments::attached(&mut conn, tool.id).await?.is_empty() {
                kept.push(ToolKey::Dependency(tool.id));
            }
        }
        for tool in tools::all(&mut conn).await? {
            let attached = !tools::attachments::of_tool(&mut conn, tool.id).await?.is_empty() || !tools::template_attachments::of_tool(&mut conn, tool.id).await?.is_empty();
            if tools::mode(&mut conn, &tool).await? == Some(Mode::Eager) && attached {
                kept.push(ToolKey::Record(tool.id));
            }
        }
        kept
    };
    for key in kept {
        held(daemon, key, Cause::Restart).await;
    }
    Ok(())
}
