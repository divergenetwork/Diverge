//! The container holding a connection: an agent, a tool, or a
//! dependency tool.

use serde::{Deserialize, Serialize};

use crate::daemon::reference;

/// One container, as the database knows it: an agent or a standalone
/// tool on record, named once and for all — by template and index —
/// since a name may be absent and the daemon answers the form that
/// always names; or a dependency tool, by the template it runs from
/// and whose it is, the owner in its once-and-for-all form, see
/// [`reference::Dependency`]. On the wire one object with one member,
/// named for the family: `{"agent":{"template":…,"index":…}}`,
/// `{"tool":{"template":…,"index":…}}` or
/// `{"dependency":{"template":…,"owner":…}}`.
///
/// A container reaches three
/// [compartments](crate::daemon::endpoints::postgres::compartments),
/// and holds a connection through one of them at a time: which is the
/// connection's own to say.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Container {
    /// An agent, as [`reference::Agent`] names one.
    Agent(reference::Agent),
    /// A tool, as [`reference::Tool`] names one.
    Tool(reference::Tool),
    /// A dependency tool, as [`reference::Dependency`] names one.
    Dependency(reference::Dependency),
}
