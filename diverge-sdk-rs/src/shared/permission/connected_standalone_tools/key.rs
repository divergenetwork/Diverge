//! One connected standalone tool, as a grant lists it.

use serde::{Deserialize, Serialize};

use crate::daemon::endpoints::daemons::DaemonId;
use crate::daemon::reference;

/// One connected standalone tool a grant reaches, by what names it
/// once and for all on the daemon that holds it: the daemon the tool
/// is on, the tool as that daemon names it, and its number among every
/// connected tool ever held from that daemon's tool. The three are
/// what its list item carries, and are given to nothing else ever, so
/// a listed tool that is let go is reached by nothing after. The
/// tool's own name here is not carried: a name is optional and may be
/// given again. No other member decodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "ConnectedStandaloneToolKey")]
#[serde(deny_unknown_fields)]
pub struct Key {
    /// The daemon the tool is on, by the ID of the holding daemon's
    /// record of it.
    pub daemon: DaemonId,
    /// The tool, as that daemon names it and as the create named it:
    /// see [`reference::Tool`]. Compared, not read.
    pub tool: reference::Tool,
    /// The tool's number among all connected tools ever held from
    /// that daemon's tool.
    pub index: u64,
}
