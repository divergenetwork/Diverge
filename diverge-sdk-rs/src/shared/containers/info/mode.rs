//! How a tool is kept running.

use serde::{Deserialize, Serialize};

/// Lazy or eager: how every tool made from an image is kept running,
/// as the image itself says. A tool's document states one, a
/// [`StandaloneTool`](super::StandaloneTool)'s and a
/// [`DependencyTool`](super::DependencyTool)'s alike, and neither
/// leaves it out: it is what the program is built to be, and no
/// template and no declaration says another. An agent's document has
/// none. Snake case on the wire: `"lazy"`, `"eager"`.
///
/// # A lazy tool runs while something holds it
///
/// Its container is started for the first thing that uses it and
/// stopped once the caller's idle time has passed with nothing
/// holding it. What holds a tool, of either mode and either kind: an
/// agent it is attached to being active, an exchange in flight into
/// it, a connect scope open on it, a file operation, a command, a
/// terminal or a frontend request in progress on it, and any scope
/// its own program has open on the caller. A dependency tool of the
/// `instance` scope is deployed, besides, when its agent's container
/// starts, and one of the `template` scope by the first thing that
/// uses it; either is then held and stopped as any tool is.
///
/// # An eager tool is held by its attachments as well
///
/// Its container is held, besides, for as long as any agent is
/// attached to it, whether or not that agent runs: started at the
/// attach that first holds it, started
/// again whenever its run ends, and started at every start of the
/// caller's while an agent is attached. Once no agent is attached it
/// is held and stopped exactly as a lazy tool is. It serves no
/// agent's run in particular, so no run's end is its end.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Run while anything holds it, stopped the caller's idle time
    /// after the last hold leaves.
    #[default]
    Lazy,
    /// Held as a lazy tool is, and as well by any agent attached to
    /// it, running or not.
    Eager,
}
