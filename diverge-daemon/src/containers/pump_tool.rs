//! A tool container's activity, off the run to its end.

use std::sync::Arc;

use diverge_sdk::daemon::tool_log as tool_log_item;
use diverge_sdk::provider::endpoints::containers::client::WaitError;
use diverge_sdk::provider::endpoints::containers::tools::run::client::execute::{Event, ExecuteStream};
use futures_util::StreamExt as _;

use super::lifecycle::End;
use super::{Ended, ToolRun, stop, tool_log};
use crate::daemon::Daemon;

/// Read the run's stream to its end: the proxy's `Active`, which
/// brackets the exchanges in flight there, is a use of the tool and
/// no hold — the daemon's own counts are what hold the run up, as
/// [`idle_tool`](super::idle_tool) states;
/// its `Inactive` is a use as well; and the stream's end or error the run over —
/// [`ended_tool`](stop::ended_tool), in a task of its own, which
/// releases what it holds — told as the end it was unless whoever
/// ended the run said first, as an agent's pump tells it: the
/// connection gone the provider lost, kept as no error; a finish the
/// run exited; any other failure kept as an `Error` item and the run
/// failed.
pub async fn pump_tool(daemon: Arc<Daemon>, run: Arc<ToolRun>, mut stream: ExecuteStream) {
    let mut ended = Ended::Finished;
    while let Some(event) = stream.next().await {
        match event {
            Ok(Event::Active | Event::Inactive) => run.touch(),
            Err(WaitError::Closed) => {
                ended = Ended::Closed;
                break;
            }
            Err(error) => {
                let words = error.to_string();
                tool_log::note(&daemon, run.id, tool_log_item::Item::Error(tool_log_item::Error { error: tool_log::words(&words) })).await;
                ended = Ended::Failed(words);
                break;
            }
        }
    }
    run.end_as(End::Unexpected(ended.reason())).await;
    stop::end_tool(daemon, run);
}
