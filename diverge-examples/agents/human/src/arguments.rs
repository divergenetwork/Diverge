//! What the agent takes as its arguments: where its desk is.

use diverge_sdk::shared::containers::request::Image;
use serde::{Deserialize, Serialize};

/// The arguments a human agent is made with.
///
/// The agent is whoever answers at its desk, and its tools are
/// whichever are attached to it. The desk is a tool container of its
/// own — the `human-desk` image — which the agent declares as its one
/// dependency, so the agent must be told which image that is. Any
/// other member is refused, so a misspelled setting is never silently
/// ignored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Arguments {
    /// The `human-desk` image the agent's desk is made from: its
    /// digest, and the references under which its bytes may be
    /// fetched. The agent declares it as a dependency tool with no
    /// arguments, no mounts and nothing beyond what the image needs.
    pub desk: Image,
}
