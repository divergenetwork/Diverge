//! One compartment, as a request names it and the daemon answers it.

use serde::{Deserialize, Serialize};

use crate::daemon::endpoints::postgres::Container;

pub use crate::container_proxy::inside::Database;

/// One compartment of the database: what a container's connection
/// reaches when its driver names `instance`, `template` or `image`.
/// Externally tagged on the wire, snake case, the member naming which
/// of the three it is:
///
/// - `{"instance":{"agent":{…}}}`, `{"instance":{"tool":{…}}}` or
///   `{"instance":{"dependency":{"template":…,"owner":{"agent":{…}}}}}`:
///   one container's own, see [`Container`].
/// - `{"template":{"kind":…,"template":…}}`: every container's made
///   from one template, the template by id.
/// - `{"image":{"digest":…}}`: every container's of one image, the
///   image by digest.
///
/// # Whose each is
///
/// | container | `instance` | `template` | `image` |
/// |---|---|---|---|
/// | an agent | the agent | its agent template | its image |
/// | a standalone tool | the tool | its tool template | its image |
/// | a dependency tool, scope `instance` | the agent and the dependency template | the dependency template | its image |
/// | a dependency tool, scope `template` | none: the connection is refused | the dependency template | its image |
///
/// A request names an agent either way, by name or by template and
/// index; the daemon answers the form that always names, template and
/// index. A filter and a grant match the answered form alone.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Compartment {
    /// One container's own: see [`Container`]. A dependency tool's
    /// owner is an agent, never `"template"`: a template-scoped tool has none.
    Instance(Container),
    /// Every container's made from one template.
    Template {
        /// What the template makes: see [`Kind`].
        kind: Kind,
        /// The template, by id.
        template: String,
    },
    /// Every container's of one image.
    Image {
        /// The image, by digest.
        digest: String,
    },
}

impl Compartment {
    /// Which of a container's three databases this is.
    pub fn database(&self) -> Database {
        match self {
            Compartment::Instance(_) => Database::Instance,
            Compartment::Template { .. } => Database::Template,
            Compartment::Image { .. } => Database::Image,
        }
    }
}

/// What kind of container a compartment is of. Snake case on the
/// wire: `"agent"`, `"standalone_tool"`, `"dependency_tool"` — the
/// `type` of an image's document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "PostgresCompartmentKind")]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// An agent's.
    Agent,
    /// A standalone tool's.
    StandaloneTool,
    /// A dependency tool's.
    DependencyTool,
}
