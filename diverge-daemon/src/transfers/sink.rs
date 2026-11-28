//! Where a transfer lands, resolved.

use std::sync::Arc;

use diverge_sdk::daemon::reference;
use diverge_sdk::daemon::transfer::Destination;

use super::Fail;
use diverge_sdk::daemon::lifecycle::Cause;

use crate::containers::{self, Opened, StartError};
use crate::daemon::Daemon;
use crate::store::{agents, tools};
use crate::volumes;

/// The destination as something that takes files.
pub enum Sink {
    /// A path in a container, opened for the operation; a tool on
    /// record has the operation in flight into it for as long as this
    /// lives.
    Opened {
        /// The container.
        opened: Opened,
        /// The path in it.
        path: Vec<String>,
    },
    /// A path in a volume, at rest.
    Volume {
        /// The volume.
        volume: reference::Volume,
        /// The path in it.
        path: Vec<String>,
    },
}

impl Sink {
    /// The destination resolved: its agent or tool — standalone or
    /// dependency — found and started, or joined, for the operation,
    /// deployed by the transfer where it was down, its volume found on
    /// its provider; `NoDestination` for one that is none.
    pub async fn resolve(daemon: &Arc<Daemon>, destination: Destination) -> Result<Sink, Fail> {
        match destination {
            Destination::Agent { agent, path } => {
                let record = {
                    let mut conn = daemon.store.acquire().await.map_err(|error| Fail::Error(error.to_string()))?;
                    agents::by_reference(&mut conn, &agent, false)
                        .await
                        .map_err(|error| Fail::Error(error.to_string()))?
                };
                let Some(record) = record else {
                    return Err(Fail::NoDestination);
                };
                let run = containers::agent(daemon, &record, Cause::Transfer).await.map_err(|error| Fail::Error(error.to_string()))?;
                Ok(Sink::Opened {
                    opened: {
                        let using = run.file_use();
                        Opened::Agent(run, using)
                    },
                    path,
                })
            }
            Destination::Tool { tool, path } => {
                let record = {
                    let mut conn = daemon.store.acquire().await.map_err(|error| Fail::Error(error.to_string()))?;
                    tools::by_reference(&mut conn, &tool, false)
                        .await
                        .map_err(|error| Fail::Error(error.to_string()))?
                };
                let Some(record) = record else {
                    return Err(Fail::NoDestination);
                };
                let run = containers::use_tool(daemon, &record, Cause::Transfer).await.map_err(|error| Fail::Error(error.to_string()))?;
                let using = run.file_use();
                Ok(Sink::Opened {
                    opened: Opened::Tool(run, Some(using)),
                    path,
                })
            }
            Destination::Dependency { dependency, path } => {
                let record = {
                    let mut conn = daemon.store.acquire().await.map_err(|error| Fail::Error(error.to_string()))?;
                    crate::store::dependency_tools::by_reference(&mut conn, &dependency, false)
                        .await
                        .map_err(|error| Fail::Error(error.to_string()))?
                };
                let Some(record) = record else {
                    return Err(Fail::NoDestination);
                };
                let run = match containers::use_dependency(daemon, record.id, Cause::Transfer).await {
                    Ok(run) => run,
                    Err(StartError::Gone) => return Err(Fail::NoDestination),
                    Err(error) => return Err(Fail::Error(error.to_string())),
                };
                Ok(Sink::Opened {
                    opened: {
                        let using = run.file_use();
                        Opened::Tool(run, Some(using))
                    },
                    path,
                })
            }
            Destination::Volume { volume, path } => {
                match volumes::find(daemon, &volume).await {
                    Ok(Some(_)) => Ok(Sink::Volume { volume, path }),
                    Ok(None) => Err(Fail::NoDestination),
                    Err(volumes::Fail::NotFound) => Err(Fail::NoDestination),
                    Err(error) => Err(Fail::from(error)),
                }
            }
        }
    }

    /// The volume at this end, if one.
    pub fn volume(&self) -> Option<&reference::Volume> {
        match self {
            Sink::Volume { volume, .. } => Some(volume),
            _ => None,
        }
    }
}
