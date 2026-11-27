//! One dependency tool, as a grant lists it.

use serde::{Deserialize, Serialize};

use crate::shared::permission::agents;

/// One dependency tool a grant reaches, by what names it once and for
/// all on the daemon that holds it: the dependency template it runs
/// from, and whose it is. One JSON object,
/// `{"template":…,"owner":…}`. No other member decodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "DependencyToolKey")]
#[serde(deny_unknown_fields)]
pub struct Key {
    /// The dependency template, by id.
    pub template: String,
    /// Whose the tool is: see [`Owner`].
    pub owner: Owner,
}

/// Whose a listed dependency tool is: the one agent it is for, or
/// nobody in particular. Externally tagged on the wire, snake case:
/// `{"agent":{"template":…,"index":…}}`, or the string `"template"`. An
/// agent is named by its template and index and never by its name,
/// which may be given again.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "DependencyToolKeyOwner")]
#[serde(rename_all = "snake_case")]
pub enum Owner {
    /// The one agent, once and for all: a tool's whose
    /// template's `scope` is `instance`.
    Agent(agents::Key),
    /// Every agent that declares the template: a tool's whose
    /// template's `scope` is `template`.
    Template,
}
