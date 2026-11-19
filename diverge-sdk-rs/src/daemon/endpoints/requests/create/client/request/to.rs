//! Whom a request is made of.

use serde::{Deserialize, Serialize};

use crate::daemon::agent_log::Identity;
use crate::daemon::reference;

/// Whom a request is made of. Snake case on the wire: the string
/// `"all"`, or `{"only": {…}}`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum To {
    /// Every account and every tool the caller's `request` grants
    /// reach, and no other. Never refused for reach.
    All,
    /// The accounts and tools named: see [`Only`].
    Only(Only),
}

/// The accounts and tools a request is made of, each named by itself
/// or by a tag. The rules:
///
/// 1. **Any of them.** A party is addressed where any member names
///    it: by itself, or by a tag it carries.
/// 2. **A kind's tags are that kind's.** `account_tags` name accounts
///    and nothing else, `standalone_tool_tags` standalone tools,
///    `dependency_tool_tags` dependency tools: a tag an account and a
///    tool both carry addresses each only through its own member.
/// 3. **A tag names every record that carries it**, those that come to
///    carry it after the create included, for as long as the request
///    is pending.
/// 4. **Something is named.** Every member empty names nobody, and the
///    create is refused as an error.
///
/// Every member may be left out, and is then empty.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Only {
    /// Accounts, each by its identity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accounts: Vec<Identity>,
    /// Every account that carries any of these tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub account_tags: Vec<String>,
    /// Standalone tools, each by its name, or its template and index.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub standalone_tools: Vec<reference::Tool>,
    /// Every standalone tool that carries any of these tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub standalone_tool_tags: Vec<String>,
    /// Dependency tools, each by its template and its owner.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependency_tools: Vec<reference::Dependency>,
    /// Every dependency tool that carries any of these tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependency_tool_tags: Vec<String>,
}

impl Only {
    /// Rule 4: whether it names nobody.
    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty()
            && self.account_tags.is_empty()
            && self.standalone_tools.is_empty()
            && self.standalone_tool_tags.is_empty()
            && self.dependency_tools.is_empty()
            && self.dependency_tool_tags.is_empty()
    }
}
