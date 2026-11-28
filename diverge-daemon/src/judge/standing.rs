//! Who is asking, as of one request.

use std::collections::HashSet;

use diverge_sdk::daemon::creator::{Client, Creator};
use diverge_sdk::daemon::grant::{self, Grant};
use diverge_sdk::shared::Identity;
use sqlx::PgConnection;

use super::{Container, ToolRef, Who};
use crate::store::{self, AgentId, ConnectedToolId, DependencyToolId, ToolId, agents, connected_standalone_tools, dependency_tools, tools};

/// Who is asking, for the length of one request: the identity they
/// are served under, every grant they hold, and — a container's — the
/// actions it holds within `attached` and what that reaches. An
/// account's grants are every grant of every role it
/// holds, read inside the request's transaction so that the judgment
/// and the write see one state; `None` is an account deleted since
/// its client was admitted, or an agent deleted since its container
/// connected, which every handler answers as `Forbidden`. A
/// container's are what its program declared at its registration,
/// fixed for the run, and what is attached to it is read as of the
/// request. The rules of `attached`:
///
/// 1. **The other side of an attachment.** The word reaches the
///    records attached to the container that asks, and nothing on the
///    container's own side.
/// 2. **A tool's is agents.** A standalone tool's, those it is
///    attached to directly and every agent of every template it is
///    attached to; a dependency tool's, those attached to its row —
///    its one agent for a tool of the `instance` scope, every agent
///    that declares it for one of the `template` scope. Over a tool
///    kind, a tool's `attached` reaches nothing.
/// 3. **An agent's is tools.** The standalone tools and the connected
///    tools attached to it directly or to the template it was made
///    from, and the dependency tools its latest registration
///    declared. Over agents, an agent's `attached` reaches nothing.
/// 4. **Held only where it reaches something.** An action within
///    `attached` is held at all — what a list, and a request judged
///    before any record is read, asks — only while at least one
///    record of that kind is attached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Standing {
    /// The account's identity, for an account; none for a container,
    /// which is no account.
    pub identity: Option<Identity>,
    /// What the requests make things as: the client, for an account;
    /// the container itself, for a container.
    pub creator: Creator,
    /// Every grant, in no order.
    grants: Vec<Grant>,
    /// The actions held within `attached`, kind by kind, in no order:
    /// a container's, and none for an account.
    attached: grant::Attached,
    /// The agents `attached` reaches: a tool's attached agents, and
    /// none for an account or an agent.
    attached_agents: HashSet<AgentId>,
    /// The standalone tools `attached` reaches: an agent's, and none
    /// for an account or a tool.
    attached_standalone_tools: HashSet<ToolId>,
    /// The connected tools `attached` reaches, likewise.
    attached_connected_standalone_tools: HashSet<ConnectedToolId>,
    /// The dependency tools `attached` reaches, likewise.
    attached_dependency_tools: HashSet<DependencyToolId>,
    /// Whether this is an administrator, which every judgment allows.
    admin: bool,
}

impl Standing {
    /// A standing that holds nothing within `attached`: an account's,
    /// or an administrator's.
    fn unattached(identity: Identity, grants: Vec<Grant>, admin: bool) -> Standing {
        Standing {
            creator: Creator::Client(Client { identity: identity.clone() }),
            identity: Some(identity),
            grants,
            attached: grant::Attached::default(),
            attached_agents: HashSet::new(),
            attached_standalone_tools: HashSet::new(),
            attached_connected_standalone_tools: HashSet::new(),
            attached_dependency_tools: HashSet::new(),
            admin,
        }
    }

    /// The standing of `who` now: an account's read fresh, a
    /// container's as its program declared it, with what is attached
    /// to it read fresh.
    pub async fn of(conn: &mut PgConnection, who: Who) -> Result<Option<Standing>, store::Error> {
        match who {
            Who::Account(id) => Ok(store::of_account(conn, id).await?.map(|(identity, grants)| Standing::unattached(identity, grants, false))),
            Who::Container(declared) => {
                let mut standing = Standing {
                    identity: None,
                    creator: declared.fixed.creator.clone(),
                    grants: declared.fixed.grants.clone(),
                    attached: declared.fixed.attached.clone(),
                    attached_agents: HashSet::new(),
                    attached_standalone_tools: HashSet::new(),
                    attached_connected_standalone_tools: HashSet::new(),
                    attached_dependency_tools: HashSet::new(),
                    admin: false,
                };
                match declared.container {
                    // Rule 2.
                    Container::Tool(ToolRef::Standalone(tool)) => {
                        standing.attached_agents = tools::attached_agents(conn, tool).await?.into_iter().collect();
                    }
                    Container::Tool(ToolRef::Dependency(tool)) => {
                        standing.attached_agents = dependency_tools::attachments::attached(conn, tool)
                            .await?
                            .into_iter()
                            .map(|attachment| attachment.agent)
                            .collect();
                    }
                    // Rule 3.
                    Container::Agent(id) => {
                        let Some(agent) = agents::by_id(conn, id, false).await? else {
                            return Ok(None);
                        };
                        standing.attached_standalone_tools = tools::attachments::of_agent(conn, id).await?.into_iter().collect();
                        standing.attached_standalone_tools.extend(tools::template_attachments::of_template(conn, &agent.template).await?);
                        standing.attached_connected_standalone_tools = connected_standalone_tools::attachments::of_agent(conn, id).await?.into_iter().collect();
                        standing.attached_connected_standalone_tools.extend(connected_standalone_tools::template_attachments::of_template(conn, &agent.template).await?);
                        standing.attached_dependency_tools = dependency_tools::of_agent(conn, id).await?.into_iter().map(|tool| tool.id).collect();
                    }
                }
                Ok(Some(standing))
            }
            Who::Admin(identity) => Ok(Some(Standing::unattached(identity, Vec::new(), true))),
        }
    }

    /// Whether this is an administrator: every judge allows it
    /// everything, with no grant read.
    pub fn admin(&self) -> bool {
        self.admin
    }

    /// Whether `action` over agents is held within `attached` at all:
    /// rule 4.
    pub fn attached_agents_hold(&self, action: grant::agents::Over) -> bool {
        !self.attached_agents.is_empty() && self.attached.agents.contains(&action)
    }

    /// Whether `action` is held within `attached` over the agent.
    pub fn attached_agent(&self, action: grant::agents::Over, agent: AgentId) -> bool {
        self.attached.agents.contains(&action) && self.attached_agents.contains(&agent)
    }

    /// Whether `action` over standalone tools is held within
    /// `attached` at all: rule 4.
    pub fn attached_standalone_tools_hold(&self, action: grant::standalone_tools::Over) -> bool {
        !self.attached_standalone_tools.is_empty() && self.attached.standalone_tools.contains(&action)
    }

    /// Whether `action` is held within `attached` over the standalone
    /// tool.
    pub fn attached_standalone_tool(&self, action: grant::standalone_tools::Over, tool: ToolId) -> bool {
        self.attached.standalone_tools.contains(&action) && self.attached_standalone_tools.contains(&tool)
    }

    /// Whether `action` over connected tools is held within
    /// `attached` at all: rule 4.
    pub fn attached_connected_standalone_tools_hold(&self, action: grant::connected_standalone_tools::Over) -> bool {
        !self.attached_connected_standalone_tools.is_empty() && self.attached.connected_standalone_tools.contains(&action)
    }

    /// Whether `action` is held within `attached` over the connected
    /// tool.
    pub fn attached_connected_standalone_tool(&self, action: grant::connected_standalone_tools::Over, tool: ConnectedToolId) -> bool {
        self.attached.connected_standalone_tools.contains(&action) && self.attached_connected_standalone_tools.contains(&tool)
    }

    /// Whether `action` over dependency tools is held within
    /// `attached` at all: rule 4.
    pub fn attached_dependency_tools_hold(&self, action: grant::dependency_tools::Over) -> bool {
        !self.attached_dependency_tools.is_empty() && self.attached.dependency_tools.contains(&action)
    }

    /// Whether `action` is held within `attached` over the dependency
    /// tool.
    pub fn attached_dependency_tool(&self, action: grant::dependency_tools::Over, tool: DependencyToolId) -> bool {
        self.attached.dependency_tools.contains(&action) && self.attached_dependency_tools.contains(&tool)
    }

    /// The grants over accounts.
    pub fn accounts(&self) -> impl Iterator<Item = &grant::accounts::Permission> {
        self.grants.iter().filter_map(|grant| match grant {
            Grant::Accounts(permission) => Some(permission),
            _ => None,
        })
    }

    /// The grants over roles.
    pub fn roles(&self) -> impl Iterator<Item = &grant::roles::Permission> {
        self.grants.iter().filter_map(|grant| match grant {
            Grant::Roles(permission) => Some(permission),
            _ => None,
        })
    }

    /// The grants over the providers.
    pub fn providers(&self) -> impl Iterator<Item = &grant::providers::Permission> {
        self.grants.iter().filter_map(|grant| match grant {
            Grant::Providers(permission) => Some(permission),
            _ => None,
        })
    }

    /// The grants through the providers, over the daemons connected to
    /// them.
    pub fn providers_daemons(&self) -> impl Iterator<Item = &grant::providers_daemons::Permission> {
        self.grants.iter().filter_map(|grant| match grant {
            Grant::ProvidersDaemons(permission) => Some(permission),
            _ => None,
        })
    }

    /// The grants through the remote daemons.
    pub fn daemons(&self) -> impl Iterator<Item = &grant::daemons::Permission> {
        self.grants.iter().filter_map(|grant| match grant {
            Grant::Daemons(permission) => Some(permission),
            _ => None,
        })
    }

    /// The grants over agent templates.
    pub fn agents_templates(&self) -> impl Iterator<Item = &grant::agents_templates::Permission> {
        self.grants.iter().filter_map(|grant| match grant {
            Grant::AgentsTemplates(permission) => Some(permission),
            _ => None,
        })
    }

    /// The grants over tool templates.
    pub fn standalone_tools_templates(&self) -> impl Iterator<Item = &grant::standalone_tools_templates::Permission> {
        self.grants.iter().filter_map(|grant| match grant {
            Grant::StandaloneToolsTemplates(permission) => Some(permission),
            _ => None,
        })
    }

    /// The grants over dependency templates.
    pub fn dependency_tools(&self) -> impl Iterator<Item = &grant::dependency_tools::Permission> {
        self.grants.iter().filter_map(|grant| match grant {
            Grant::DependencyTools(permission) => Some(permission),
            _ => None,
        })
    }

    /// The grants over agents.
    pub fn agents(&self) -> impl Iterator<Item = &grant::agents::Permission> {
        self.grants.iter().filter_map(|grant| match grant {
            Grant::Agents(permission) => Some(permission),
            _ => None,
        })
    }

    /// The grants over tools.
    pub fn standalone_tools(&self) -> impl Iterator<Item = &grant::standalone_tools::Permission> {
        self.grants.iter().filter_map(|grant| match grant {
            Grant::StandaloneTools(permission) => Some(permission),
            _ => None,
        })
    }

    /// The grants over connected tools: other daemons' standalone tools,
    /// held here.
    pub fn connected_standalone_tools(&self) -> impl Iterator<Item = &grant::connected_standalone_tools::Permission> {
        self.grants.iter().filter_map(|grant| match grant {
            Grant::ConnectedStandaloneTools(permission) => Some(permission),
            _ => None,
        })
    }

    /// The grants over the database.
    pub fn postgres(&self) -> impl Iterator<Item = &grant::postgres::Permission> {
        self.grants.iter().filter_map(|grant| match grant {
            Grant::Postgres(permission) => Some(permission),
            _ => None,
        })
    }

    /// The grants over volumes.
    pub fn volumes(&self) -> impl Iterator<Item = &grant::volumes::Permission> {
        self.grants.iter().filter_map(|grant| match grant {
            Grant::Volumes(permission) => Some(permission),
            _ => None,
        })
    }

    /// The grants over the database's compartments.
    pub fn postgres_compartments(&self) -> impl Iterator<Item = &grant::postgres_compartments::Permission> {
        self.grants.iter().filter_map(|grant| match grant {
            Grant::PostgresCompartments(permission) => Some(permission),
            _ => None,
        })
    }

    /// The grants over the events.
    pub fn events(&self) -> impl Iterator<Item = &grant::events::Permission> {
        self.grants.iter().filter_map(|grant| match grant {
            Grant::Events(permission) => Some(permission),
            _ => None,
        })
    }

    /// Every grant over images the standing holds.
    pub fn images(&self) -> impl Iterator<Item = &grant::images::Permission> {
        self.grants.iter().filter_map(|grant| match grant {
            Grant::Images(permission) => Some(permission),
            _ => None,
        })
    }

    /// Every grant over the host's identities the standing holds.
    pub fn identities(&self) -> impl Iterator<Item = &grant::identities::Permission> {
        self.grants.iter().filter_map(|grant| match grant {
            Grant::Identities(permission) => Some(permission),
            _ => None,
        })
    }
}
