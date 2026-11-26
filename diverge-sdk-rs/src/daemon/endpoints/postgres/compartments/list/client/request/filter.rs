//! What narrows a list of compartments.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::daemon::endpoints::postgres::compartments::{Compartment, Database, Kind};

/// The filter over compartments: every member optional, and every one
/// given a condition a compartment passes or does not. In a [list
/// request](super::Frame) it is flattened into the request and narrows
/// what the daemon sends; in a grant's `within` it is how far the
/// grant reaches. A filter with no member given passes every
/// compartment.
///
/// # Any one of, every one of
///
/// A member that lists candidates matches a compartment that is any
/// one of them; `all_tags` one that carries every one of them,
/// `any_tags` one that carries any one of them. An empty list is
/// absent, and matches every compartment.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "PostgresCompartmentsFilter")]
pub struct Filter {
    /// Exactly these, each as the daemon answers it: an agent by its
    /// template and index, never by name. Absent when empty, and then
    /// any compartment.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub compartments: Vec<Compartment>,
    /// Any one of these of a container's three: `"instance"`,
    /// `"template"`, `"image"`. Absent when empty, and then any.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub databases: Vec<Database>,
    /// Of any one of these kinds of container: see [`Kind`]. Absent
    /// when empty, and then any kind.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kinds: Vec<Kind>,
    /// Of any one of these templates, by id: a `template`
    /// compartment's own, an agent's or a tool's `instance`'s, a
    /// dependency tool's `instance`'s dependency template. Absent when
    /// empty, and then any.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub templates: Vec<String>,
    /// Of any one of these images, by digest: an `image`
    /// compartment's own. Absent when empty, and then any.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<String>,
    /// Whether a
    /// [`delete`](crate::daemon::endpoints::postgres::compartments::delete)
    /// would be allowed now, connections apart — `true` — or not,
    /// `false`; absent, either.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deletable: Option<bool>,
    /// Every one of these among the compartment's tags. Absent when
    /// empty, and then any tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub all_tags: Vec<String>,
    /// Any one of these among the compartment's tags; with
    /// `all_tags`, both hold. Absent when empty, and then any tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub any_tags: Vec<String>,
    /// On record at or after this instant. On the wire an RFC 3339
    /// timestamp. Absent, no lower bound.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_from: Option<DateTime<Utc>>,
    /// On record at or before this instant. Absent, no upper bound.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_to: Option<DateTime<Utc>>,
}

/// As a grant's `within`, the one record-naming member is
/// `compartments`: see [`Only`](crate::shared::permission::Only).
impl crate::shared::permission::Only for Filter {
    fn mixed(&self) -> Option<&'static str> {
        crate::shared::permission::mixed(&[!self.compartments.is_empty()], !self.all_tags.is_empty() || !self.any_tags.is_empty())
    }
}
