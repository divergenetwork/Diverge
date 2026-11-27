//! One request a requester put to the daemon, as its log keeps it.

use serde::{Deserialize, Serialize};

/// One request: where it went, what it said, and whether the daemon
/// let it through. See the [rules](super).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct Command {
    /// The request's path, as the daemon names it: `agents/message`,
    /// `agents/templates/create`, `logs`.
    pub path: String,
    /// The request's body, a template's arguments replaced (rule 2).
    pub request: serde_json::Value,
    /// Whether the daemon let it through (rule 3).
    pub allowed: bool,
}
