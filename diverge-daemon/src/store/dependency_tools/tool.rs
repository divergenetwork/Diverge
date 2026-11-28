//! One dependency tool, as the store holds it.

use chrono::{DateTime, Utc};
use diverge_sdk::daemon::creator::Creator;
use diverge_sdk::daemon::endpoints::dependency_tools::list::server::response::{Attached, Listed, Run, Shown};
use diverge_sdk::daemon::key;
use diverge_sdk::shared::containers::dependencies::Template;
use diverge_sdk::shared::containers::info::{self, Mode, Scope};
use diverge_sdk::shared::permission::Grant;

use crate::store::agents::Agent;
use crate::store::{AgentId, DependencyToolId};

/// A dependency tool row.
#[derive(Debug, Clone, PartialEq)]
pub struct DependencyTool {
    /// The row.
    pub id: DependencyToolId,
    /// The template it is of, by id.
    pub template: String,
    /// Its number among the dependency tools of the template, once
    /// and for all.
    pub index: u64,
    /// The template, whole.
    pub declared: Template,
    /// The document of the template's image: what the image says of
    /// itself, which the template does not — how the tool is kept
    /// running, and the memory and the disk it needs.
    pub document: info::DependencyTool,
    /// How far it is shared: what its scope key is made of, as its
    /// image's `scope` said when the tool was made.
    pub scope: Scope,
    /// The one agent it is for, when its scope is per agent.
    pub owner_agent: Option<AgentId>,
    /// Its tags, sorted bytewise.
    pub tags: Vec<String>,
    /// What its container declared it may ask of the daemon when it
    /// last started; empty for one that declared none or has never
    /// started.
    pub permissions: Vec<Grant>,
    /// When it was made.
    /// When its container last came up or went down.
    pub last_deployed: Option<DateTime<Utc>>,
    /// When its calls and reads in flight last began or ceased.
    pub last_active: Option<DateTime<Utc>>,
    pub created: DateTime<Utc>,
    /// The agent whose registration made it.
    pub creator: Creator,
}

impl DependencyTool {
    /// Whether it is an eager tool, kept by the daemon, rather than a
    /// lazy tool, run while it is used.
    pub fn is_eager(&self) -> bool {
        self.document.mode == Mode::Eager
    }

    /// The memory and the disk its container is to have, in bytes, in
    /// that order: what its image needs and what its template adds,
    /// summed. A sum past what a limit can say is the reason, in a
    /// sentence.
    pub fn limits(&self) -> Result<(u64, u64), String> {
        let memory = self.document.memory.checked_add(self.declared.extra_memory).ok_or("the image's memory and the template's extra_memory are together more than a limit can be")?;
        let disk = self.document.disk.checked_add(self.declared.extra_disk).ok_or("the image's disk and the template's extra_disk are together more than a limit can be")?;
        Ok((memory, disk))
    }

    /// Whether it is one agent's own: a lazy tool whose scope is
    /// `instance`, deployed at its agent's registration, and the one
    /// kind of tool that may mount its agent's files. Every other
    /// tool is an eager one, kept by the daemon while any agent is
    /// attached, or a lazy one that is many agents', deployed with
    /// each agent that declares it and by any use that finds it down.
    pub fn own(&self) -> bool {
        !self.is_eager() && self.scope == Scope::Instance
    }

    /// Whose it is, as the wire names it: the owner agent, handed in
    /// by the caller for a per-agent tool since a name is as it is
    /// called now, or nobody. `None` for a per-agent tool whose owner
    /// the caller did not hand in.
    pub fn owner(&self, owner_agent: Option<&Agent>) -> Option<key::Owner> {
        Some(match self.scope {
            Scope::Instance => key::Owner::Agent(owner_agent?.key()),
            Scope::Template => key::Owner::Template,
        })
    }

    /// The tool as a key: its template and whose it is.
    pub fn key(&self, owner_agent: Option<&Agent>) -> Option<key::Tool> {
        Some(key::Tool::Dependency {
            template: self.template.clone(),
            owner: self.owner(owner_agent)?,
        })
    }

    /// The tool as a list reports it, with its owner, the agents
    /// attached to it, whether it is working now and its run, loaded by
    /// the caller, and its template's arguments only where `arguments`
    /// says the caller asked for them and may see them.
    pub fn report(&self, owner: key::Owner, agents: Vec<Attached>, active: bool, run: Option<Run>, arguments: bool) -> Listed {
        Listed {
            id: self.template.clone(),
            owner,
            mode: self.document.mode,
            scope: self.scope,
            index: self.index,
            created: self.created,
            creator: self.creator.clone(),
            tags: self.tags.clone(),
            template: Shown::of(&self.declared, arguments),
            agents,
            active,
            last_active: self.last_active,
            deployed: run.is_some(),
            last_deployed: self.last_deployed,
            run,
            permissions: self.permissions.clone(),
        }
    }
}

/// The scope's word, as the `scope` column holds it: the snake case
/// of the wire.
pub fn scope_name(scope: Scope) -> &'static str {
    match scope {
        Scope::Instance => "instance",
        Scope::Template => "template",
    }
}

/// The scope a column's word names, or `None` for a word that is
/// neither.
pub fn scope_of(name: &str) -> Option<Scope> {
    match name {
        "instance" => Some(Scope::Instance),
        "template" => Some(Scope::Template),
        _ => None,
    }
}
