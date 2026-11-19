//! Whether a request is made of a party.

use std::sync::Arc;

use diverge_sdk::daemon::creator::{self, Creator};
use diverge_sdk::daemon::endpoints::requests::create::client::request::To;
use diverge_sdk::daemon::grant;
use diverge_sdk::daemon::reference;
use sqlx::PgConnection;

use crate::daemon::Daemon;
use crate::judge::{self, Standing};
use crate::requests::Party;
use crate::serve::{dependency_tools, standalone_tools};
use crate::store::{self, accounts, agents, dependency_tools as dependencies, tools};

/// Whether any of `named` is among `carried`: a party is addressed by
/// a tag where it carries one of those named.
fn tagged(named: &[String], carried: &[String]) -> bool {
    named.iter().any(|tag| carried.contains(tag))
}

/// Whether the standalone tool reference names the tool: by its name,
/// or by its template and its index.
fn names(reference: &reference::Tool, tool: &tools::Tool) -> bool {
    match reference {
        reference::Tool::Name { name } => tool.name.as_deref() == Some(name.as_str()),
        reference::Tool::TemplateIndex { template, index } => tool.template == *template && tool.index == *index,
    }
}

/// Whether the request made `to` those, by a creator of `standing`, is
/// one `party` is sent. The rules:
///
/// 1. **Addressed.** `all` addresses every party. Otherwise the party
///    is named by itself — an account by its identity, a standalone
///    tool by its name or its template and index, a dependency tool by
///    its template and its owner — or carries a tag its own kind's
///    member names. An administrator is an account, addressed by its
///    identity and by the tags it carries.
/// 2. **Reached.** The creator's `request` grant for the party's kind
///    reaches it, as that grant's reach is read everywhere else: an
///    account by `accounts`, a standalone tool by `standalone_tools`,
///    a dependency tool by `dependency_tools`. A listener is
///    connected, and is judged as connected.
/// 3. **Both, as of now.** The party's record is read here, so a tag
///    it gained or lost, and a row that is gone, are as they are at
///    the asking. A party with no record is sent nothing.
pub async fn eligible(conn: &mut PgConnection, daemon: &Arc<Daemon>, party: &Party, to: &To, standing: &Standing) -> Result<bool, store::Error> {
    Ok(match party {
        Party::Account(id) => {
            let Some(account) = accounts::by_id(conn, *id).await? else {
                return Ok(false);
            };
            let addressed = match to {
                To::All => true,
                To::Only(only) => only.accounts.contains(&account.identity) || tagged(&only.account_tags, &account.tags),
            };
            addressed && judge::accounts::over(standing, grant::accounts::Over::Request, &account, true)
        }
        Party::Admin(identity) => {
            let tags = store::admins::tags(conn, identity, false).await?;
            let addressed = match to {
                To::All => true,
                To::Only(only) => only.accounts.contains(identity) || tagged(&only.account_tags, &tags),
            };
            addressed && judge::accounts::over_admin(standing, grant::accounts::Over::Request, identity, &tags, true)
        }
        Party::Standalone(id) => {
            let Some(tool) = tools::by_id(conn, *id, false).await? else {
                return Ok(false);
            };
            let addressed = match to {
                To::All => true,
                To::Only(only) => only.standalone_tools.iter().any(|reference| names(reference, &tool)) || tagged(&only.standalone_tool_tags, &tool.tags),
            };
            addressed && standalone_tools::reaches(conn, daemon, standing, grant::standalone_tools::Over::Request, &tool).await?
        }
        Party::Dependency(id) => {
            let Some(tool) = dependencies::by_id(conn, *id, false).await? else {
                return Ok(false);
            };
            let addressed = match to {
                To::All => true,
                To::Only(only) => {
                    let owner = owner(conn, &tool).await?;
                    only.dependency_tools.iter().any(|reference| reference.template == tool.template && owner.as_ref().is_some_and(|owner| judge::filter::dependency_tools::owned_by(owner, &reference.owner)))
                        || tagged(&only.dependency_tool_tags, &tool.tags)
                }
            };
            addressed && dependency_tools::reaches(conn, daemon, standing, grant::dependency_tools::Over::Request, &tool).await?
        }
    })
}

/// A dependency tool's owner, as a reference names one: `None` for a
/// per-agent tool whose agent is gone.
async fn owner(conn: &mut PgConnection, tool: &dependencies::DependencyTool) -> Result<Option<diverge_sdk::daemon::key::Owner>, store::Error> {
    let agent = match tool.owner_agent {
        Some(id) => agents::by_id(conn, id, false).await?,
        None => None,
    };
    Ok(tool.owner(agent.as_ref()))
}

/// How `party` is named to a creator — in a delivery, and as who
/// answered: an account or an administrator as the client it is, by
/// `standing`'s own word; a tool as the tool it is, by its record,
/// never as whoever made it. `None` for a tool whose row, or whose
/// owner, is gone.
pub async fn named(conn: &mut PgConnection, party: &Party, standing: &Standing) -> Result<Option<Creator>, store::Error> {
    Ok(match party {
        Party::Account(_) | Party::Admin(_) => Some(standing.creator.clone()),
        Party::Standalone(id) => tools::by_id(conn, *id, false).await?.map(|tool| Creator::Tool(tool.snapshot())),
        Party::Dependency(id) => {
            let Some(tool) = dependencies::by_id(conn, *id, false).await? else {
                return Ok(None);
            };
            owner(conn, &tool).await?.map(|owner| {
                Creator::Tool(creator::Tool::Dependency {
                    template: tool.template.clone(),
                    owner,
                })
            })
        }
    })
}

/// Every pending request `party` has not been sent, judged, and each
/// that is made of it delivered: what a listener does when it opens,
/// and again when its own kind's records change. A request delivered
/// by its creator's own look in the same moment is delivered once:
/// see [`Requests`](crate::requests::Requests).
pub async fn catch_up(daemon: &Arc<Daemon>, party: &Party) -> Result<(), store::Error> {
    let candidates = daemon.live.requests.candidates(party).await;
    if candidates.is_empty() {
        return Ok(());
    }
    let mut conn = daemon.store.acquire().await?;
    let mut made_of = Vec::new();
    for candidate in &candidates {
        if eligible(&mut conn, daemon, party, &candidate.to, &candidate.standing).await? {
            made_of.push(&candidate.id);
        }
    }
    drop(conn);
    for id in made_of {
        daemon.live.requests.deliver(id, party).await;
    }
    Ok(())
}
