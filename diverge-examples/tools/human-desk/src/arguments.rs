//! What the desk takes as its arguments: nothing.

use serde::{Deserialize, Serialize};

/// The arguments a human desk is made with.
///
/// None: the desk is its one agent's, and the person at it is whoever
/// opens its frontend. `{}` is the one value; any member is refused,
/// so a misspelled setting is never silently ignored.
///
/// # How it runs
///
/// As a dependency tool of the `human` agent: an eager tool whose
/// scope is `instance`, so each agent has a desk of its own, kept up
/// while the agent declares it, with the page's sockets held between
/// the agent's turns.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Arguments {}
