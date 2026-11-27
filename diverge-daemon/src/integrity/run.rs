//! The pass itself.

use std::path::{Path, PathBuf};

use chrono::Utc;
use futures_util::StreamExt as _;
use futures_util::stream;

use super::{BOOKS, Closed, Error, admins, close, owned};
use crate::containers::ToolKey;
use crate::logs;
use crate::store::{self, AgentId, ConnectedToolId, DependencyToolId, Store, ToolId, agents, connected_standalone_tools, dependency_tools, tools};

/// How many log folders are mended at once: each holds two files
/// open, and a daemon may keep many thousands of logs.
const AT_ONCE: usize = 16;

/// One log folder to mend, and whose it is.
#[derive(Debug)]
enum Folder {
    /// An agent's: a run's log.
    Agent(AgentId),
    /// A tool's, of any of the three kinds: a run's log.
    Tool(ToolKey),
    /// A command log, an account's or an administrator's: it holds no
    /// run, and only its files are made whole.
    Commands(PathBuf),
}

/// The daemon's state under `logs` and in `store` made whole: see
/// [`integrity`](super) for the rules and their order.
pub async fn run(store: &Store, logs: &Path) -> Result<(), Error> {
    let mut folders = Vec::new();
    {
        // Rule 1, book after book on the one connection: each is one
        // query and one listing.
        let mut conn = store.acquire().await?;
        for (folder, table) in BOOKS {
            for id in owned(&mut conn, logs, folder, table).await? {
                folders.push(match folder {
                    "agents" => Folder::Agent(AgentId(id)),
                    "tools" => Folder::Tool(ToolKey::Record(ToolId(id))),
                    "dependency_tools" => Folder::Tool(ToolKey::Dependency(DependencyToolId(id))),
                    "connected_standalone_tools" => Folder::Tool(ToolKey::Connected(ConnectedToolId(id))),
                    _ => Folder::Commands(logs.join(folder).join(id.to_string())),
                });
            }
        }
    }
    folders.extend(admins(logs).await?.into_iter().map(Folder::Commands));
    // Rules 2 to 4 over the folders, and rule 5 beside them: neither
    // waits on the other.
    let mended = async {
        let mut mending = stream::iter(folders).map(|folder| mend(store, logs, folder)).buffer_unordered(AT_ONCE);
        while let Some(mended) = mending.next().await {
            mended?;
        }
        Ok::<(), Error>(())
    };
    mended.await?;
    Ok(())
}

/// One folder: its files made whole, then — a run's log — the run it
/// still holds deployed closed, and its record told. In that order,
/// since a run is closed by appending to files that are whole.
async fn mend(store: &Store, logs: &Path, folder: Folder) -> Result<(), Error> {
    use crate::logs::Book as _;
    match folder {
        Folder::Commands(dir) => Ok(logs::repair(&dir).await?),
        Folder::Agent(id) => {
            logs::repair(&id.dir(logs)).await?;
            let Some(Closed { provider, active }) = close(logs, id).await? else {
                return Ok(());
            };
            let now = Utc::now();
            let mut conn = store.acquire().await?;
            agents::set_deployed(&mut conn, id, &provider, now).await?;
            if active {
                agents::set_active(&mut conn, id, now).await?;
            }
            Ok(())
        }
        Folder::Tool(key) => {
            logs::repair(&key.dir(logs)).await?;
            let Some(Closed { provider, active }) = close(logs, key).await? else {
                return Ok(());
            };
            let now = Utc::now();
            let mut conn = store.acquire().await?;
            told(&mut conn, key, &provider, now, active).await?;
            Ok(())
        }
    }
}

/// A tool's record told its run went down at `now`, and stopped
/// working then where it was `active`: the kind's own columns, a tool
/// on record alone keeping the provider.
async fn told(
    conn: &mut sqlx::PgConnection,
    key: ToolKey,
    provider: &diverge_sdk::shared::Identity,
    now: chrono::DateTime<Utc>,
    active: bool,
) -> Result<(), store::Error> {
    match key {
        ToolKey::Record(id) => {
            tools::set_deployed(conn, id, provider, now).await?;
            if active {
                tools::set_active(conn, id, now).await?;
            }
        }
        ToolKey::Dependency(id) => {
            dependency_tools::set_deployed(conn, id, now).await?;
            if active {
                dependency_tools::set_active(conn, id, now).await?;
            }
        }
        ToolKey::Connected(id) => {
            connected_standalone_tools::set_deployed(conn, id, now).await?;
            if active {
                connected_standalone_tools::set_active(conn, id, now).await?;
            }
        }
    }
    Ok(())
}
