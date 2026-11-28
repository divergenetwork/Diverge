//! A request the agent's container put to the daemon, as the log
//! keeps it.

use serde::{Deserialize, Serialize};

use crate::daemon::command_log;

/// One request the agent's own container put to the daemon, over its
/// daemon connection, under the permissions its program declared.
/// `type` is the string `command`, and the command's members — its
/// path, its body, whether it was let through — lie beside it: see
/// [`command_log::Command`], and
/// [`command_log`](crate::daemon::command_log) for the rules it is
/// kept by. It moves nothing in the run's life: a command makes the
/// agent neither active nor inactive.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct Command {
    /// Always `command`.
    pub r#type: CommandType,
    /// The command, its members the item's.
    #[serde(flatten)]
    pub command: command_log::Command,
}

/// [`Command`]'s discriminator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[derive(schemars::JsonSchema)]
pub enum CommandType {
    #[serde(rename = "command")]
    #[default]
    Command,
}
