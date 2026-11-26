//! What keeps a compartment from being deleted.

use std::collections::HashSet;

use diverge_sdk::daemon::endpoints::postgres::compartments::{Compartment, Kind};
use sqlx::PgConnection;

use super::Entry;
use crate::store::Error;

/// What there is, now, that a compartment a delete of its own removes
/// would be kept for: loaded once, for one compartment or for all of
/// them. The rules of [`deletable`](Holds::deletable):
///
/// 1. **An `instance` compartment is never deletable.** It goes with
///    the agent or the tool it is of.
/// 2. **An agent's or a standalone tool's `template` compartment is
///    never deletable.** It goes with the template.
/// 3. **A dependency template's `template` compartment is deletable
///    while no agent is attached to any tool of the template**: no
///    agent's last registration declared it.
/// 4. **An agent's image's `image` compartment is deletable while no
///    agent template there is contains the image**; a standalone
///    tool's likewise, of tool templates. A deleted template contains
///    nothing.
/// 5. **A dependency tool's image's `image` compartment is deletable
///    while no agent is attached to any tool of any dependency
///    template of the image.**
#[derive(Debug, Clone, Default)]
pub struct Holds {
    /// The dependency templates some agent is attached to a tool of.
    dependency_templates: HashSet<String>,
    /// The images of those templates, by digest.
    dependency_images: HashSet<String>,
    /// The images of the agent templates there are, by digest.
    agent_images: HashSet<String>,
    /// The images of the tool templates there are, by digest.
    tool_images: HashSet<String>,
}

impl Holds {
    /// What there is now.
    pub async fn load(conn: &mut PgConnection) -> Result<Holds, Error> {
        let dependency_templates: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT t.template FROM diverge.dependency_attachments a JOIN diverge.dependency_tools t ON t.id = a.tool",
        )
        .fetch_all(&mut *conn)
        .await?;
        let dependency_images: Vec<Option<String>> = sqlx::query_scalar(
            "SELECT DISTINCT d.template->'image'->>'digest' FROM diverge.dependency_attachments a \
             JOIN diverge.dependency_tools t ON t.id = a.tool JOIN diverge.dependencies d ON d.id = t.template",
        )
        .fetch_all(&mut *conn)
        .await?;
        let agent_images: Vec<Option<String>> =
            sqlx::query_scalar("SELECT DISTINCT template->'image'->>'digest' FROM diverge.agents_templates WHERE NOT deleted")
                .fetch_all(&mut *conn)
                .await?;
        let tool_images: Vec<Option<String>> =
            sqlx::query_scalar("SELECT DISTINCT template->'image'->>'digest' FROM diverge.tools_templates WHERE NOT deleted")
                .fetch_all(&mut *conn)
                .await?;
        Ok(Holds {
            dependency_templates: dependency_templates.into_iter().collect(),
            dependency_images: dependency_images.into_iter().flatten().collect(),
            agent_images: agent_images.into_iter().flatten().collect(),
            tool_images: tool_images.into_iter().flatten().collect(),
        })
    }

    /// Whether a delete of the compartment would be allowed now,
    /// connections apart: the rules above.
    pub fn deletable(&self, entry: &Entry) -> bool {
        match (&entry.compartment, entry.kind) {
            // Rule 1.
            (Compartment::Instance(_), _) => false,
            // Rule 2.
            (Compartment::Template { .. }, Kind::Agent | Kind::StandaloneTool) => false,
            // Rule 3.
            (Compartment::Template { template, .. }, Kind::DependencyTool) => !self.dependency_templates.contains(template),
            // Rule 4.
            (Compartment::Image { digest }, Kind::Agent) => !self.agent_images.contains(digest),
            (Compartment::Image { digest }, Kind::StandaloneTool) => !self.tool_images.contains(digest),
            // Rule 5.
            (Compartment::Image { digest }, Kind::DependencyTool) => !self.dependency_images.contains(digest),
        }
    }
}
