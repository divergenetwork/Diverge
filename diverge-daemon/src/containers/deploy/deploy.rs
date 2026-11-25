//! Deploying every dependency, at once: each on record under its
//! owner, the agent's own lazy tools started, the shared lazy tools
//! served idle, the eager tools kept.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use diverge_sdk::shared::mcp::attestation;
use diverge_sdk::daemon::creator::{self, Creator};
use diverge_sdk::daemon::key;
use diverge_sdk::provider::endpoints::containers::tools::run::client::{execute as tools_run, request as tools_request};
use diverge_sdk::shared::containers::dependencies::Template;
use diverge_sdk::shared::containers::info::{self, Mode, Scope};
use diverge_sdk::shared::Identity;
use diverge_sdk::shared::error::Error;
use futures_util::future;
use tokio::sync::{Mutex, watch};

use crate::containers::answerers::Answerer;
use crate::containers::images::{self, Pulls};
use crate::containers::fuse::Mounts;
use crate::containers::lifecycle::End;
use crate::containers::capable::Needs;
use crate::containers::mcp::Served;
use crate::containers::eager;
use crate::containers::{Inflight, Key, ToolHandle, ToolKey, ToolRun, build, permissions, provider, pump_tool, secrets, stop};
use crate::daemon::{Daemon, Kind};
use crate::serve::inner;
use crate::judge::ToolRef;
use crate::store::agents::{self, Agent};
use crate::store::dependency_tools::{self, DependencyTool};
use crate::store::{self, AgentId, DependencyToolId, hash};

/// Deploy every template for the agent `answerer` answers for, whose
/// container runs under `id` on the answerer's provider.
///
/// The rules:
///
/// 1. Every template is checked before anything is recorded or
///    started: one declared twice — one id twice — one that names a
///    mount under a scope that is not `instance`, one whose secrets
///    are not names, or not names the agent binds, and one whose
///    image has no document the daemon can read, or an agent's, is
///    refused.
/// 2. In one transaction: every template's body on record, every
///    tool of it under the owner its `scope` names — the agent
///    itself, its agent template, or nobody — made or found made, the
///    agent attached to each as of now, and detached from every tool
///    it no longer declares.
/// 3. A lazy tool whose scope is `instance` is the agent's run's:
///    every one is started at once, in a task of its own, waited for
///    (see [`start_all`]), and served to the agent under its
///    template's id once every one is up. A registration cancelled
///    midway orphans none.
/// 4. An eager tool is kept up, its hold recomputed, and its run
///    waited for — live already, or its supervisor's next start up —
///    and served the same way.
/// 5. A lazy tool whose scope is `template` or `global` is many
///    agents': it is not started here. It is served idle, and the
///    first use starts it — see
///    [`use_shared`](crate::containers::tools::use_shared).
/// 6. Every tool the agent left is told its attachments changed.
///
/// A tool container asking, an agent gone, a store that refuses the
/// records, no provider to run on, a path of the agent's that cannot
/// be served, a provider that will not run one — every candidate
/// tried — or an eager tool whose start fails, is the error, naming
/// the dependency by its id where there is one, and every tool
/// started under rule 3 is stopped.
pub async fn deploy(answerer: &Answerer, id: String, templates: Vec<Template>) -> Result<(), Error> {
    let (Key::Agent(agent), Some(attestation::Caller::Agent(caller))) = (answerer.key, &answerer.caller) else {
        return Err(error("", "a tool container declares no dependencies"));
    };
    let agent_key = &caller.key;
    let creator = Creator::Agent(creator::Agent {
        template: agent_key.template.clone(),
        index: agent_key.index,
        name: agent_key.name.clone(),
    });
    let daemon = &answerer.daemon;
    let row = {
        let mut conn = daemon.store.acquire().await.map_err(|failure| error("", &failure.to_string()))?;
        agents::by_id(&mut conn, agent, false).await.map_err(|failure| error("", &failure.to_string()))?
    };
    let Some(row) = row else {
        return Err(error("", "the agent is gone"));
    };
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
        let document = match holds(&template, document, &row) {
            Ok(document) => document,
            Err(reason) => return Err(error(&template_id, &reason)),
        };
        held.push((template_id, template, document));
    }
    let declared = held;
    let (tools, left) = record(daemon, agent, agent_key, &creator, &declared)
        .await
        .map_err(|reason| error("", &reason))?;
    let owner = Owner {
        daemon: Arc::clone(daemon),
        provider: answerer.provider.clone(),
        inflight: Arc::clone(&answerer.inflight),
        files: Arc::clone(&answerer.files),
        served: Arc::clone(&answerer.served),
        agent: row.clone(),
        key: agent_key.clone(),
        container: id,
    };
    let own: Vec<DependencyTool> = tools.iter().filter(|tool| tool.runs_with_agent()).cloned().collect();
    let runs = match tokio::spawn(start_all(owner, own)).await {
        Ok(Ok(runs)) => runs,
        Ok(Err((template_id, reason))) => return Err(error(&template_id, &reason)),
        Err(failure) => return Err(error("", &failure.to_string())),
    };
    let mut kept = Vec::new();
    for tool in &tools {
        if !tool.is_eager() {
            continue;
        }
        let key = ToolKey::Dependency(tool.id);
        eager::keep(daemon, key).await;
        eager::held(daemon, key).await;
        if let Err(reason) = up(daemon, key).await {
            {
                let mut served = answerer.served.lock().await;
                for run in &runs {
                    served.remove_lazy(run);
                }
            }
            stop::stop_dependencies(daemon, runs, End::Expected).await;
            return Err(error(&tool.template, &reason));
        }
        kept.push(tool.id);
    }
    for id in left {
        eager::held(daemon, ToolKey::Dependency(id)).await;
    }
    *answerer.kept.lock().await = Some(kept);
    let mut served = answerer.served.lock().await;
    for tool in &tools {
        if tool.is_eager() {
            served.insert_eager(tool.id, &tool.template, &short(&tool.template));
        } else if !tool.runs_with_agent() {
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
/// 3. **Every secret the declaration names is one the image states,
///    named once, and one the agent binds**: an image states every
///    secret its program could take, a declaration names those this
///    tool needs, and the value of each is taken through the agent's
///    own binding of the name.
/// 4. **What the image needs and what the declaration adds is a
///    limit**: neither sum is past what one can say.
///
/// The reason it is not, in a sentence.
fn holds(template: &Template, document: info::Document, agent: &Agent) -> Result<info::DependencyTool, String> {
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
    inner::declared(&template.secrets)?;
    if let Some(name) = template.secrets.iter().find(|name| !document.secrets.iter().any(|secret| &secret.name == *name)) {
        return Err(format!("the secret {name} is none the tool's image states"));
    }
    secrets::bind(&template.secrets, &agent.secrets)?;
    // Rule 4.
    document.memory.checked_add(template.extra_memory).ok_or("the image's memory and the template's extra_memory are together more than a limit can be")?;
    document.disk.checked_add(template.extra_disk).ok_or("the image's disk and the template's extra_disk are together more than a limit can be")?;
    Ok(document)
}

/// What a dependency's start takes of the agent — its answerer's and
/// its record's — owned, so the starts run in a task of their own.
struct Owner {
    /// The daemon.
    daemon: Arc<Daemon>,
    /// The provider the agent's container runs on: where its paths are
    /// served from.
    provider: Identity,
    /// The agent's exchanges in flight: a dependency's hold the agent up.
    inflight: Arc<Inflight>,
    /// The agent's file uses: a dependency's hold the agent up.
    files: Arc<Inflight>,
    /// The agent's served set, the lazy tools served into it.
    served: Arc<Mutex<Served>>,
    /// The agent's record: its secrets' binding.
    agent: Agent,
    /// The agent's key.
    key: key::Agent,
    /// The agent's container's id.
    container: String,
}

/// Every lazy tool started at once, then every one served to the
/// agent under its template's id, all in the one task: the first that
/// could not start is the error — its template's id and the reason —
/// and every one started is stopped. Served before the deploy goes on,
/// a run is the agent's from here: the agent's start stops it with an
/// attempt that fails, and the agent's end with the run.
async fn start_all(owner: Owner, own: Vec<DependencyTool>) -> Result<Vec<Arc<ToolRun>>, (String, String)> {
    let started = future::join_all(own.iter().map(|tool| async {
        let outcome = start(&owner, tool).await;
        (tool.template.clone(), outcome)
    }))
    .await;
    let mut runs = Vec::with_capacity(started.len());
    let mut failed = None;
    for (template_id, outcome) in started {
        match outcome {
            Ok(run) => runs.push(run),
            Err(reason) => failed = failed.or(Some((template_id, reason))),
        }
    }
    if let Some(failed) = failed {
        stop::stop_dependencies(&owner.daemon, runs, End::Expected).await;
        return Err(failed);
    }
    let mut served = owner.served.lock().await;
    for run in &runs {
        if let Some(tool) = &run.dependency {
            served.insert_lazy(Arc::clone(run), &short(&tool.template));
        }
    }
    Ok(runs)
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
    agent_key: &key::Agent,
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
                agent_template: &agent_key.template,
                creator,
            },
        )
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

/// One lazy tool started: its secrets bound through the agent's
/// binding, to be taken at its own registration and held under the
/// dependency tool as holder, its mounts served from the agent, its
/// container run on the first candidate provider that will, what its
/// program declared it may ask of the daemon kept on its row, its run
/// entered as live under its row with its pump — a run still live
/// under the row, from an agent run that ended without it, stopped
/// first, since a row has one run. The reason it could not be, in a
/// sentence; a provider that refuses after the registration gives
/// back what it took, and a failure once the run is made undoes it as
/// [`undo_tool`](stop::undo_tool) states.
async fn start(owner: &Owner, tool: &DependencyTool) -> Result<Arc<ToolRun>, String> {
    let daemon = &owner.daemon;
    let agent_key = &owner.key;
    if !tool.runs_with_agent() {
        return Err("only a lazy tool of the `instance` scope is started with its agent".to_string());
    }
    let (fuse_file_mounts, fuse_directory_mounts) = (&tool.declared.fuse_file_mounts, &tool.declared.fuse_directory_mounts);
    let limits = tool.limits()?;
    let key = ToolKey::Dependency(tool.id);
    if let Some(stale) = daemon.live.tool_run(key).await {
        stop::stop_dependencies(daemon, vec![stale], End::Expected).await;
    }
    let needs = Needs {
        volume_modes: Vec::new(),
        image: Some(tool.declared.image.clone()),
        memory: limits.0,
        disk: limits.1,
    };
    let candidates = provider::candidates(daemon, None, &needs).await.map_err(|error| error.to_string())?;
    let binding = secrets::bind(&tool.declared.secrets, &owner.agent.secrets)?;
    let mounts = Arc::new(
        Mounts::build_dependency(daemon, &owner.provider, &owner.container, fuse_file_mounts, fuse_directory_mounts)
            .await
            .map_err(|error| error.to_string())?,
    );
    let container = build::dependency(&tool.template, &tool.declared, limits, &mounts).await;
    let image = container.image.clone();
    let attestation = attestation::Tool::Dependency {
        image: image.clone(),
        template: tool.template.clone(),
        owner: key::Owner::Agent(agent_key.clone()),
    };
    let sender = Creator::Tool(creator::Tool::Dependency {
        template: tool.template.clone(),
        owner: key::Owner::Agent(agent_key.clone()),
    });
    let touched = watch::channel(Instant::now()).0;
    let mut last = String::new();
    for (identity, handle) in candidates {
        let run_answerer = Arc::new(Answerer {
            daemon: Arc::clone(daemon),
            key: Key::Tool(key),
            caller: None,
            sender: sender.clone(),
            tool: Some(ToolRef::Dependency(tool.id)),
            permissions: Mutex::new(Vec::new()),
            provider: identity.clone(),
            mounts: Arc::clone(&mounts),
            served: Arc::new(Mutex::new(Served::new())),
            pulls: Arc::new(Pulls::new()),
            touched: touched.clone(),
            inflight: Arc::clone(&owner.inflight),
            files: Arc::clone(&owner.files),
            kept: Mutex::new(None),
            declared: tool.declared.secrets.clone(),
            binding: binding.clone(),
            secrets: Mutex::new(None),
        });
        let opened = tools_run::execute(&handle, &tools_request::Frame(container.clone()), run_answerer.set()).await;
        // The run is answered: its pull is over either way.
        run_answerer.pulls.release().await;
        let (container_id, handle, stream) = match opened {
            Ok(opened) => opened,
            Err(error) => {
                secrets::abandon(daemon, &run_answerer).await;
                last = format!("{error:?}");
                continue;
            }
        };
        let permissions = run_answerer.permissions.lock().await.clone();
        let run = Arc::new(ToolRun {
            id: key,
            attestation: attestation.clone(),
            sender: sender.clone(),
            provider: identity,
            container: Some(container_id.id),
            handle: ToolHandle::Run(handle),
            eager: false,
            connections: Mutex::new(HashSet::new()),
            held: watch::channel(false).0,
            inflight: Arc::new(Inflight::new()),
            // The agent's: a use on the dependency holds the agent up.
            files: Arc::clone(&owner.files),
            touched: touched.clone(),
            mounts: Arc::clone(&mounts),
            volumes: Vec::new(),
            ended: watch::channel(false).0,
            tasks: Mutex::new(Vec::new()),
            edits: Mutex::new(()),
            dependency: Some(tool.clone()),
            sessions: Mutex::new(HashMap::new()),
            secrets: Mutex::new(run_answerer.secrets.lock().await.take()),
            epoch: daemon.live.epoch().await,
            log: daemon.live.tool_log(&daemon.store, key).await,
            ending: Mutex::new(None),
        });
        // From here a failure is the run's undoing.
        if let Err(failure) = permissions::record(daemon, ToolRef::Dependency(tool.id), &permissions).await {
            stop::undo_tool(daemon, run, false).await;
            return Err(failure.to_string());
        }
        crate::containers::lifecycle::deployed(daemon.logs.clone(), run.id, Arc::clone(&run.log), run.epoch, run.provider.clone()).await;
        if let Ok(mut conn) = daemon.store.acquire().await {
            let _ = crate::store::dependency_tools::set_deployed(&mut conn, tool.id, chrono::Utc::now()).await;
        }
        daemon.live.insert_tool(Arc::clone(&run)).await;
        daemon.live.changed(Kind::DependencyTools);
        daemon.live.nudge_capacity(&run.provider);
        // Spawned and kept under the tasks' lock, while the run is still
        // the live one: an end meanwhile drains it after.
        {
            let mut tasks = run.tasks.lock().await;
            if daemon.live.is_tool_run(&run).await {
                tasks.push(tokio::spawn(pump_tool::pump_tool(Arc::clone(daemon), Arc::clone(&run), stream)).abort_handle());
            }
        }
        return Ok(run);
    }
    mounts.stop().await;
    Err(format!("no provider ran it; the last said: {last}"))
}

/// The eager tool up: its run live now, or its supervisor's next start
/// waited for and up. Why it is not, in a sentence: the start's own
/// words, or an eager tool not kept. The last word the supervisor said
/// is marked read before the run is looked for, so a start that came
/// up between the two is seen live — a run is live before its
/// outcome is said — and an outcome said before this registration,
/// a failure under another agent's binding, is not read: the next is
/// the start this registration causes, its attachment now the
/// latest, under the binding that counts.
async fn up(daemon: &Daemon, key: ToolKey) -> Result<(), String> {
    let Some(state) = daemon.live.eager_state(key).await else {
        return Err("the eager tool is not kept".to_string());
    };
    let mut started = state.started.clone();
    started.borrow_and_update();
    if daemon.live.tool_run(key).await.is_some() {
        return Ok(());
    }
    if started.changed().await.is_err() {
        return Err("the eager tool is not kept any more".to_string());
    }
    let outcome = (*started.borrow_and_update()).clone();
    match outcome {
        Some(eager::Outcome::Failed(reason)) => Err(reason),
        _ => Ok(()),
    }
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
