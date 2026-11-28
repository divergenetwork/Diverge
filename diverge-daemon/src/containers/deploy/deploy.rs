//! Deploying every dependency, at once: each on record under its
//! owner, the agent's own lazy tools started, the shared lazy tools
//! served idle, the eager tools kept.

use std::collections::HashSet;
use std::sync::Arc;

use diverge_sdk::shared::mcp::attestation;
use diverge_sdk::daemon::creator::{self, Creator};
use diverge_sdk::daemon::lifecycle::Cause;
use diverge_sdk::shared::containers::dependencies::Template;
use diverge_sdk::shared::containers::info::{self, Mode, Scope};
use diverge_sdk::shared::error::Error;
use futures_util::future;

use super::{Mounted, tool};
use crate::containers::answerers::Answerer;
use crate::containers::images;
use crate::containers::lifecycle::End;
use crate::containers::eager;
use crate::containers::{Key, ToolKey, stop};
use crate::daemon::{Daemon, Kind};
use crate::store::agents;
use crate::store::dependency_tools::{self, DependencyTool};
use crate::store::{self, AgentId, DependencyToolId, hash};

/// Deploy every template for the agent `answerer` answers for, whose
/// container runs under `id` on the answerer's provider.
///
/// The rules:
///
/// 1. Every template is checked before anything is recorded or
///    started: one declared twice — one id twice — one that names a
///    mount under a scope that is not `instance`, and one whose
///    image has no document the daemon can read, or an agent's, is
///    refused.
/// 2. In one transaction: every template's body on record, every
///    tool of it under the owner its `scope` names — the agent
///    itself, its agent template, or nobody — made or found made, the
///    agent attached to each as of now, and detached from every tool
///    it no longer declares.
/// 3. A lazy tool whose scope is `instance` is the agent's own:
///    every one is deployed now, at once, in a task of its own, waited
///    for (see [`start_all`]), and served to the agent under its
///    template's id once every one is up. One that mounts the agent's
///    files is deployed against this container, a run of it left from
///    an earlier one stopped first; one that mounts nothing and is
///    still up is left as it is. A registration cancelled midway
///    orphans none.
/// 4. An eager tool is kept up, its attachments read anew, and its run
///    waited for — live already, or its supervisor's next start up —
///    and served the same way.
/// 5. A lazy tool whose scope is `template` is many
///    agents': it is not started here. It is served idle, and the
///    first use starts it — see
///    [`use_dependency`](crate::containers::tools::use_dependency).
/// 6. Every tool the agent left is told its attachments changed.
///
/// What is deployed here is held and undeployed afterwards as every
/// tool is: an agent that goes on to sit idle may find a tool of its
/// own gone down, and its next loop brings it back.
///
/// A tool container asking, an agent gone, a store that refuses the
/// records, no provider to run on, a path of the agent's that cannot
/// be served, a provider that will not run one — every candidate
/// tried — or an eager tool whose start fails, is the error, naming
/// the dependency by its id where there is one, and every tool
/// started under rule 3 that mounts the agent's files is stopped.
pub async fn deploy(answerer: &Answerer, id: String, templates: Vec<Template>) -> Result<(), Error> {
    let (Key::Agent(agent), Some(attestation::Caller::Agent(caller))) = (answerer.key, &answerer.caller) else {
        return Err(error("", "a tool container declares no dependencies"));
    };
    let creator = Creator::Agent(creator::Agent {
        template: caller.key.template.clone(),
        index: caller.key.index,
        name: caller.key.name.clone(),
    });
    let daemon = &answerer.daemon;
    let row = {
        let mut conn = daemon.store.acquire().await.map_err(|failure| error("", &failure.to_string()))?;
        agents::by_id(&mut conn, agent, false).await.map_err(|failure| error("", &failure.to_string()))?
    };
    if row.is_none() {
        return Err(error("", "the agent is gone"));
    }
    let mut declared = Vec::with_capacity(templates.len());
    let mut ids = HashSet::new();
    for template in templates {
        let template_id = hash::template_id(&template.hashed()).map_err(|failure| error("", &failure.to_string()))?;
        if !ids.insert(template_id.clone()) {
            return Err(error(&template_id, "declared twice"));
        }
        declared.push((template_id, template));
    }
    // Every dependency's image has a document, found as
    // `images::document` finds it and left on record: asked of all of
    // them at once, since no one's answer bears on another's. The
    // first, in the order declared, that has none, or whose document
    // will not do, refuses the declaration before anything is
    // recorded or started.
    let documents = future::join_all(declared.iter().map(|(_, template)| images::document(daemon, &template.image))).await;
    let mut held = Vec::with_capacity(declared.len());
    for ((template_id, template), document) in declared.into_iter().zip(documents) {
        let document = match document {
            Ok(Ok(document)) => document,
            Ok(Err(reason)) => return Err(error(&template_id, &reason)),
            Err(failure) => return Err(error(&template_id, &failure.to_string())),
        };
        let document = match holds(&template, document) {
            Ok(document) => document,
            Err(reason) => return Err(error(&template_id, &reason)),
        };
        held.push((template_id, template, document));
    }
    let declared = held;
    let (tools, left) = record(daemon, agent, &creator, &declared)
        .await
        .map_err(|reason| error("", &reason))?;
    let mounted = Mounted {
        provider: answerer.provider.clone(),
        container: id,
        files: Arc::clone(&answerer.files),
        touched: answerer.touched.clone(),
    };
    let own: Vec<DependencyTool> = tools.iter().filter(|tool| tool.own()).cloned().collect();
    let owned: Vec<DependencyToolId> = own.iter().map(|tool| tool.id).collect();
    match tokio::spawn(start_all(Arc::clone(daemon), mounted, own)).await {
        Ok(Ok(())) => {}
        Ok(Err((template_id, reason))) => return Err(error(&template_id, &reason)),
        Err(failure) => return Err(error("", &failure.to_string())),
    }
    let mut kept = Vec::new();
    for tool in &tools {
        if !tool.is_eager() {
            continue;
        }
        let key = ToolKey::Dependency(tool.id);
        eager::held(daemon, key, Cause::Registration).await;
        if let Err(reason) = eager::up(daemon, key).await {
            stop::stop_mounted(daemon, &owned, End::Expected).await;
            return Err(error(&tool.template, &reason));
        }
        kept.push(tool.id);
    }
    for id in left {
        eager::held(daemon, ToolKey::Dependency(id), Cause::Attachment).await;
    }
    *answerer.kept.lock().await = Some(kept);
    let mut served = answerer.served.lock().await;
    for tool in &tools {
        if tool.is_eager() {
            served.insert_eager(tool.id, &tool.template, &short(&tool.template));
        } else if tool.own() {
            // A lazy tool of the agent's own: deployed above.
            served.insert_own(tool.id, &tool.template, &short(&tool.template));
        } else {
            // A lazy tool shared with other agents: served idle, and
            // started by the first use.
            served.insert_shared(tool.id, &tool.template, &short(&tool.template));
        }
    }
    daemon.live.changed(Kind::DependencyTools);
    daemon.live.changed(Kind::StandaloneTools);
    Ok(())
}

/// Whether a declaration is one the daemon deploys, by its image's
/// document: the image's own, a dependency tool's, where it is. The
/// rules, in order:
///
/// 1. **The image is a dependency tool's.** An agent's image, or a
///    standalone tool's, is no dependency: a standalone tool is made
///    by hand from a tool template, and by nothing else.
/// 2. **A mount is a lazy tool's of one agent.** A declaration that
///    names a mount of the agent's files or directories is of an image
///    whose `mode` is `lazy` and whose `scope` is `instance`: of any
///    other there is no one agent whose files they are, or no run of
///    its that the tool's run is.
/// 3. **What the image needs and what the declaration adds is a
///    limit**: neither sum is past what one can say.
///
/// The reason it is not, in a sentence.
fn holds(template: &Template, document: info::Document) -> Result<info::DependencyTool, String> {
    // Rule 1.
    let document = match document {
        info::Document::DependencyTool(document) => document,
        info::Document::Agent(_) => return Err("the image's document is an agent's, and a dependency is a dependency tool's image".to_string()),
        info::Document::StandaloneTool(_) => {
            return Err("the image's document is a standalone tool's, which is made from a tool template and is no agent's dependency".to_string());
        }
    };
    // Rule 2.
    if template.mounts() && !(document.mode == Mode::Lazy && document.scope == Scope::Instance) {
        return Err("a tool mounts the agent's files only where its image's mode is `lazy` and its scope is `instance`".to_string());
    }
    // Rule 3.
    document.memory.checked_add(template.extra_memory).ok_or("the image's memory and the template's extra_memory are together more than a limit can be")?;
    document.disk.checked_add(template.extra_disk).ok_or("the image's disk and the template's extra_disk are together more than a limit can be")?;
    Ok(document)
}

/// Every lazy tool of the agent's own deployed at once, all in the one
/// task, against the agent's container `mounted`. The rules:
///
/// 1. **A tool that mounts the agent's files** is deployed against
///    this container: a run of it still live — left from the agent's
///    earlier container, whose paths it mounts — is stopped first,
///    since a row has one run.
/// 2. **A tool that mounts nothing** and is still up is left as it
///    is; one that is down is deployed alone.
/// 3. **The first that could not be deployed is the error** — its
///    template's id and the reason — and every one deployed here
///    that mounts the agent's files is stopped, since the agent's
///    start fails with it.
async fn start_all(daemon: Arc<Daemon>, mounted: Mounted, own: Vec<DependencyTool>) -> Result<(), (String, String)> {
    let started = future::join_all(own.iter().map(|one| async {
        let outcome = if one.declared.mounts() {
            // Rule 1.
            if let Some(stale) = daemon.live.tool_run(ToolKey::Dependency(one.id)).await {
                stop::stop_dependencies(&daemon, vec![stale], End::Expected).await;
            }
            tool(&daemon, one, Some(mounted.clone()), Cause::Registration).await
        } else {
            // Rule 2.
            tool(&daemon, one, None, Cause::Registration).await
        };
        (one.template.clone(), outcome.map(|_| ()).map_err(|error| error.to_string()))
    }))
    .await;
    // Rule 3.
    let failed = started.into_iter().find_map(|(template_id, outcome)| outcome.err().map(|reason| (template_id, reason)));
    if let Some(failed) = failed {
        let ids: Vec<DependencyToolId> = own.iter().map(|one| one.id).collect();
        stop::stop_mounted(&daemon, &ids, End::Expected).await;
        return Err(failed);
    }
    Ok(())
}

/// Every declared template on record — its body, and its tool under
/// the agent's owner, made by this registration or found made — the
/// agent attached to each, as of now, and detached from every tool it
/// did not declare this time, in one transaction. The tools, in the
/// order declared, and the tools left. The reason the store refused,
/// in a sentence.
async fn record(
    daemon: &Daemon,
    agent: AgentId,
    creator: &Creator,
    declared: &[(String, Template, info::DependencyTool)],
) -> Result<(Vec<DependencyTool>, Vec<DependencyToolId>), String> {
    let mut tx = daemon.store.begin().await.map_err(|error| error.to_string())?;
    let mut tools = Vec::with_capacity(declared.len());
    for (template_id, template, document) in declared {
        store::dependencies::create(&mut tx, template_id, template)
            .await
            .map_err(|error| error.to_string())?;
        // How far the tool is shared is its image's to say.
        let scope = document.scope;
        let tool = dependency_tools::find_or_create(
            &mut tx,
            &dependency_tools::Declared {
                template: template_id,
                scope,
                agent,
                creator,
            },
        )
        .await
        .map_err(|error| error.to_string())?;
        crate::database::register(&mut tx, crate::containers::Key::Tool(crate::containers::ToolKey::Dependency(tool.id)))
            .await
            .map_err(|error| error.to_string())?;
        dependency_tools::attachments::attach(&mut tx, tool.id, agent)
            .await
            .map_err(|error| error.to_string())?;
        tools.push(tool);
    }
    let ids: Vec<DependencyToolId> = tools.iter().map(|tool| tool.id).collect();
    let left = dependency_tools::attachments::detach_except(&mut tx, agent, &ids)
        .await
        .map_err(|error| error.to_string())?;
    tx.commit().await.map_err(|error| error.to_string())?;
    Ok((tools, left))
}

/// The first eight characters of a template id: what a dependency is
/// served to its agent as.
pub fn short(template_id: &str) -> String {
    template_id.chars().take(8).collect()
}

/// The run's error, naming the dependency by its template's id.
fn error(template_id: &str, reason: &str) -> Error {
    Error(serde_json::json!({
        "kind": "dependency",
        "dependency": template_id,
        "error": reason,
    }))
}
