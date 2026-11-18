//! One standalone tool, as a grant lists it.

use serde::{Deserialize, Serialize};

/// One standalone tool a grant reaches, by what names it once and for
/// all on the daemon that holds it: the template it was made from and
/// its number among every tool that daemon ever made from that
/// template. The pair is given to nothing else ever, so a listed tool
/// that is deleted is reached by nothing after. No name is carried: a
/// name is optional and may be given again, and a grant must not come
/// to reach another tool by one being renamed. No other member
/// decodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "StandaloneToolKey")]
#[serde(deny_unknown_fields)]
pub struct Key {
    /// The template the tool was made from, by id.
    pub template: String,
    /// The tool's number among all tools the daemon ever made from
    /// that template.
    pub index: u64,
}
