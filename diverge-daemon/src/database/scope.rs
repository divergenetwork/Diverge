//! Which compartments are a container's, and what each is called.

use diverge_sdk::container_proxy::inside::Database;
use diverge_sdk::daemon::endpoints::postgres::Container;
use diverge_sdk::daemon::endpoints::postgres::compartments::{Compartment, Kind};
use diverge_sdk::daemon::reference;
use diverge_sdk::shared::canonical;
use diverge_sdk::shared::containers::info::Scope;
use sha2::{Digest, Sha256};
use sqlx::PgConnection;

use crate::containers::{Key, ToolKey};
use crate::store::agents::Agent;
use crate::store::dependency_tools::DependencyTool;
use crate::store::tools::Tool;
use crate::store::{self, agents, agents_templates, dependency_tools, tools, tools_templates};

/// What one container reaches: itself as the database names it, what
/// kind it is, and its compartments. The rules:
///
/// 1. **An agent's.** `instance` is the agent; `template` its agent
///    template; `image` its template's image.
/// 2. **A standalone tool's.** `instance` is the tool; `template` its
///    tool template; `image` its template's image.
/// 3. **A dependency tool's whose scope is `instance`.** `instance` is
///    the tool itself, the dependency template under its one agent;
///    `template` the dependency template, whichever agent declares
///    it; `image` the template's image.
/// 4. **A dependency tool's whose scope is `global`.** No `instance`:
///    it runs for no one agent. `template` and `image` as rule 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reached {
    /// The container, an agent in it by its template and index.
    pub container: Container,
    /// What kind of container it is.
    pub kind: Kind,
    /// Its own compartment; none by rule 4.
    pub instance: Option<Compartment>,
    /// Its template's.
    pub template: Compartment,
    /// Its image's.
    pub image: Compartment,
}

impl Reached {
    /// The compartment the database names, or `None` by rule 4.
    pub fn of(&self, database: Database) -> Option<&Compartment> {
        match database {
            Database::Instance => self.instance.as_ref(),
            Database::Template => Some(&self.template),
            Database::Image => Some(&self.image),
        }
    }

    /// Every compartment the container reaches.
    pub fn all(&self) -> impl Iterator<Item = &Compartment> {
        self.instance.iter().chain([&self.template, &self.image])
    }
}

/// The role, and the schema, of the compartment: `diverge_`, then the
/// first forty hexadecimal characters of the SHA-256 of the
/// compartment's canonical bytes, an agent in it by its template and
/// index. Forty-eight bytes of `[a-z0-9_]`, stable across restarts and
/// providers, never reused because what the compartment is of is never
/// reused, safe to write as a bare identifier, and computable from the
/// compartment at any time, which is how a delete finds what to drop.
pub fn role_of(compartment: &Compartment) -> String {
    let bytes = canonical::bytes(compartment).unwrap_or_default();
    let digest = hex::encode(Sha256::digest(&bytes));
    format!("diverge_{}", &digest[..40])
}

/// The key the compartment's provisioning and its drop are serialized
/// under: the first eight bytes of the SHA-256 of the role, as a
/// signed integer.
pub fn lock_key(role: &str) -> i64 {
    let digest = Sha256::digest(role.as_bytes());
    i64::from_be_bytes(digest[..8].try_into().unwrap_or([0; 8]))
}

/// The agent as the database names it.
pub fn container_of_agent(agent: &Agent) -> Container {
    Container::Agent(reference::Agent::TemplateIndex {
        template: agent.template.clone(),
        index: agent.index,
    })
}

/// The tool as the database names it.
pub fn container_of_tool(tool: &Tool) -> Container {
    Container::Tool(reference::Tool::TemplateIndex {
        template: tool.template.clone(),
        index: tool.index,
    })
}

/// The dependency tool as the database names it: by its template's id
/// and its one agent — `owner`, loaded by the caller — or everyone, as
/// its scope says. `None` for a per-agent tool whose agent the caller
/// did not hand in.
pub fn container_of_dependency_tool(tool: &DependencyTool, owner: Option<&Agent>) -> Option<Container> {
    let owner = match tool.scope {
        Scope::Instance => {
            let owner = owner?;
            reference::Owner::Agent(reference::Agent::TemplateIndex {
                template: owner.template.clone(),
                index: owner.index,
            })
        }
        Scope::Global => reference::Owner::Global,
    };
    Some(Container::Dependency(reference::Dependency {
        template: tool.template.clone(),
        owner,
    }))
}

/// The agent's own compartment: rule 1 of [`Reached`].
pub fn instance_of_agent(agent: &Agent) -> Compartment {
    Compartment::Instance(container_of_agent(agent))
}

/// The tool's own compartment: rule 2.
pub fn instance_of_tool(tool: &Tool) -> Compartment {
    Compartment::Instance(container_of_tool(tool))
}

/// The dependency tool's own compartment: rule 3, and `None` by rule
/// 4, or for a per-agent tool whose agent the caller did not hand in.
pub fn instance_of_dependency_tool(tool: &DependencyTool, owner: Option<&Agent>) -> Option<Compartment> {
    match tool.scope {
        Scope::Instance => container_of_dependency_tool(tool, owner).map(Compartment::Instance),
        Scope::Global => None,
    }
}

/// The compartment of every container made from the template.
pub fn template_of(kind: Kind, template: &str) -> Compartment {
    Compartment::Template {
        kind,
        template: template.to_string(),
    }
}

/// The container the key names, as the database names it: a record's
/// if its record is there, a dependency tool's if its row is. A
/// connected tool has none: its container is another daemon's, and
/// reaches no database here.
pub async fn container_of(conn: &mut PgConnection, key: Key) -> Result<Option<Container>, store::Error> {
    Ok(reached(conn, key).await?.map(|reached| reached.container))
}

/// What the container the key names reaches, by the rules of
/// [`Reached`]. `None` for a record that is gone, or whose template
/// is, and for a connected tool.
pub async fn reached(conn: &mut PgConnection, key: Key) -> Result<Option<Reached>, store::Error> {
    Ok(match key {
        Key::Agent(id) => {
            let Some(agent) = agents::by_id(conn, id, false).await? else {
                return Ok(None);
            };
            let Some(template) = agents_templates::by_id(conn, &agent.template, false).await? else {
                return Ok(None);
            };
            Some(Reached {
                container: container_of_agent(&agent),
                kind: Kind::Agent,
                instance: Some(instance_of_agent(&agent)),
                template: template_of(Kind::Agent, &agent.template),
                image: Compartment::Image {
                    digest: template.template.image.digest.clone(),
                },
            })
        }
        Key::Tool(ToolKey::Record(id)) => {
            let Some(tool) = tools::by_id(conn, id, false).await? else {
                return Ok(None);
            };
            let Some(template) = tools_templates::by_id(conn, &tool.template, false).await? else {
                return Ok(None);
            };
            Some(Reached {
                container: container_of_tool(&tool),
                kind: Kind::StandaloneTool,
                instance: Some(instance_of_tool(&tool)),
                template: template_of(Kind::StandaloneTool, &tool.template),
                image: Compartment::Image {
                    digest: template.template.image.digest.clone(),
                },
            })
        }
        Key::Tool(ToolKey::Connected(_)) => None,
        Key::Tool(ToolKey::Dependency(id)) => {
            let Some(tool) = dependency_tools::by_id(conn, id, false).await? else {
                return Ok(None);
            };
            let owner = match tool.owner_agent {
                Some(id) => agents::by_id(conn, id, false).await?,
                None => None,
            };
            let Some(container) = container_of_dependency_tool(&tool, owner.as_ref()) else {
                return Ok(None);
            };
            Some(Reached {
                container,
                kind: Kind::DependencyTool,
                instance: instance_of_dependency_tool(&tool, owner.as_ref()),
                template: template_of(Kind::DependencyTool, &tool.template),
                image: Compartment::Image {
                    digest: tool.declared.image.digest.clone(),
                },
            })
        }
    })
}

/// The compartment as the daemon names it: an agent a request named
/// by name, by its template and index. `None` where the request names
/// an agent or a tool that is not there.
pub async fn canonical_of(conn: &mut PgConnection, compartment: &Compartment) -> Result<Option<Compartment>, store::Error> {
    let Compartment::Instance(container) = compartment else {
        return Ok(Some(compartment.clone()));
    };
    let container = match container {
        Container::Agent(agent) => agents::by_reference(conn, agent, false).await?.map(|agent| container_of_agent(&agent)),
        Container::Tool(tool) => tools::by_reference(conn, tool, false).await?.map(|tool| container_of_tool(&tool)),
        Container::Dependency(reference::Dependency { template, owner }) => match owner {
            reference::Owner::Agent(agent) => agents::by_reference(conn, agent, false).await?.map(|agent| {
                Container::Dependency(reference::Dependency {
                    template: template.clone(),
                    owner: reference::Owner::Agent(reference::Agent::TemplateIndex {
                        template: agent.template.clone(),
                        index: agent.index,
                    }),
                })
            }),
            reference::Owner::Global => Some(container.clone()),
        },
    };
    Ok(container.map(Compartment::Instance))
}
