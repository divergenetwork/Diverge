//! Which of its two modes a dependency template has.

use serde::{Deserialize, Serialize};

/// Lazy or eager: the `mode` a dependency template's
/// [`Mode`](crate::shared::containers::dependencies::Mode) is tagged
/// with, as a value of its own to narrow a list by. Snake case on the
/// wire, as the tag is: `"lazy"`, `"eager"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// A template whose tools run while they are used.
    Lazy,
    /// A template whose tools are kept up for as long as any agent is
    /// attached to them.
    Eager,
}
