//! Which record of its template a container is.

use serde::{Deserialize, Serialize};

use crate::daemon::key::Owner;

/// Which of the records made from its
/// [`template`](super::Container::template) a container is: the
/// caller's word, beside the template's id, for which one this is.
/// The two together are the record's own name — an agent's or a
/// standalone tool's template and index, a dependency tool's template
/// and owner — and so what the container's program names itself by in
/// whatever it asks of the caller. The provider hands it to the
/// container beside the template, at its registration, and reads
/// nothing of it.
///
/// Externally tagged, snake case, and laid flat beside `template`
/// wherever it is carried: `"index":3`, or `"owner":"template"`, or
/// `"owner":{"agent":{"template":…,"index":…}}`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Which {
    /// An agent's or a standalone tool's: its number among all the
    /// records the caller ever made from the template, as its list
    /// item carries it.
    Index(u64),
    /// A dependency tool's: whose it is — the one agent it is for, or
    /// every agent that declares the template. See [`Owner`].
    Owner(Owner),
}
