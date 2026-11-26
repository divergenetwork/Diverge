//! What the taskboard takes as its arguments: nothing.

use serde::{Deserialize, Serialize};

/// The arguments a taskboard tool is made with.
///
/// None: the board is the tool's database, and the tool is the same
/// whoever calls it. `{}` is the one value; any member is refused, so
/// a misspelled setting is never silently ignored.
///
/// # How it runs
///
/// As a dependency tool: an eager tool, declared by the image of every
/// agent that works from the board. The board is kept in the
/// tool's `template` database, so every agent that declares this
/// taskboard works from the one board, whatever template it is made from.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Arguments {}
