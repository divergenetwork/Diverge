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
/// # A lazy tool runs while it is used
///
/// Its container is started for the first thing that uses it and
/// stopped when the last leaves. What uses a standalone tool is an
/// agent it is attached to beginning a loop, a connect scope opened on
/// it, a file operation. A dependency tool's is by its
/// [`Scope`](super::Scope): of the `instance` scope it is the agent's
/// run's, deployed when the agent's container starts and stopped with
/// it; of the `template` scope it is one container for
/// every agent the scope covers, started for the first exchange an
/// attached agent sends it and stopped once the caller's idle time has
/// passed with nothing using it.
///
/// # An eager tool is kept while anything holds it
///
/// Its container is kept running for as long as any agent is attached
/// to it, or, of a standalone tool, any connect scope is open on it:
/// started at the attach or the connect that first holds it, started
/// again whenever its run ends, and started at every start of the
/// caller's while an agent is attached. Once nothing holds it, it is
/// retired after the caller's idle time, an attachment within that
/// time keeping it. It serves no agent's run in particular, so no
/// run's end is its end.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Run while used, stopped when the last user leaves.
    #[default]
    Lazy,
    /// Kept running while anything holds it, retired the caller's idle
    /// time after the last leaves.
    Eager,
}
