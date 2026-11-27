//! What a tool template says of itself beside what every template
//! says: that it is one.

use serde::{Deserialize, Serialize};

/// `"type":"tool"`, and nothing beside it: what tells a tool template
/// from an agent template with the same image, limits and arguments.
/// Flat among the template's members: a template whose `type` is
/// anything else does not decode as a tool template. How every tool
/// made from the template is kept running is not here: it is the
/// image's own to say, the `mode` of its
/// [document](crate::shared::containers::info::StandaloneTool).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(tag = "type", rename = "tool")]
pub struct ToolType {}

/// A tool template: the daemon's one
/// [`Template`](crate::daemon::template::Template), typed
/// [`ToolType`].
pub type Template = crate::daemon::template::Template<ToolType>;

/// A tool template as a get or a list answers it: the daemon's one
/// [`Shown`](crate::daemon::template::Shown), typed [`ToolType`].
pub type Shown = crate::daemon::template::Shown<ToolType>;
