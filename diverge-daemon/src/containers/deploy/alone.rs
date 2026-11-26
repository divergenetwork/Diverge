//! One run of a dependency tool that stands alone of any agent's run.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use diverge_sdk::shared::mcp::attestation;
use diverge_sdk::daemon::creator::{self, Creator};
use diverge_sdk::provider::endpoints::containers::tools::run::client::{execute as tools_run, request as tools_request};
use diverge_sdk::shared::containers::info::Scope;
use indexmap::IndexMap;
use tokio::sync::{Mutex, watch};

use crate::containers::answerers::Answerer;
use crate::containers::images::Pulls;
use crate::containers::fuse::Mounts;
use crate::containers::capable::Needs;
use crate::containers::mcp::Served;
use crate::containers::start::Starting;
use crate::containers::{Inflight, Key, StartError, ToolHandle, ToolKey, ToolRun, build, idle, permissions, provider, pump_tool, secrets, undo_tool};
use crate::daemon::{Daemon, Kind};
use crate::judge::ToolRef;
use crate::store::agents;
use crate::store::dependency_tools::{DependencyTool, attachments};

/// The run of a dependency tool that is no one agent's run's — an
/// eager tool, of any scope, or a lazy tool whose scope is `template`
/// or `global` — started now if it was not up.
///
/// The rules:
///
/// 1. One start at a time per tool: a second, meanwhile, waits and
///    finds the run the first made.
/// 2. The container is the template's image, limits and arguments,
///    with no mount: the tool is not one agent's, and mounts nothing
///    of any one's.
/// 3. Its secrets are bound by name through the agent that spawns the
///    run, taken at the tool's own registration and held under the
///    dependency tool as holder. That agent is the tool's owner, for
///    an `instance` scope, and otherwise the attached agent whose
///    registration declared the template last — so the binding may
///    differ from run to run, and a vault key bound anew is taken at
///    the next.
/// 4. It runs on the first candidate provider that will, under the
///    grants its program declared at its registration, kept on its
///    row, its run entered as live under its row with its pump.
/// 5. A lazy tool's run has the idle clock a lazy standalone tool's
///    has — [`idle_tool`](crate::containers::idle::idle_tool) — and
///    ends `idle_seconds` after its last use with nothing in flight.
///    An eager tool's has none: its keeping is its supervisor's.
/// 6. A per-agent tool whose agent is gone cannot be named and is not
///    started; one that declares secrets with no agent attached is
///    not started either. A provider that refuses after the
///    registration gives back what it took, and a failure once the
///    run is made undoes it as
///    [`undo_tool`](crate::containers::undo_tool) states.
///
/// The start runs in a task of its own, waited for: a caller
/// cancelled midway neither tears it nor orphans what it opened.
pub async fn alone(daemon: &Arc<Daemon>, tool: &DependencyTool) -> Result<Arc<ToolRun>, StartError> {
    let (daemon, tool) = (Arc::clone(daemon), tool.clone());
    let started = tokio::spawn(async move {
        // Rule 1.
        let _starting = Starting::hold(&daemon, Key::Tool(ToolKey::Dependency(tool.id))).await;
        start_alone(&daemon, &tool).await
    });
    started.await.unwrap_or_else(|error| Err(StartError::Unfinished(error.to_string())))
}

async fn start_alone(daemon: &Arc<Daemon>, tool: &DependencyTool) -> Result<Arc<ToolRun>, StartError> {
    let key = ToolKey::Dependency(tool.id);
    if let Some(run) = daemon.live.tool_run(key).await {
        return Ok(run);
    }
    let (owner_agent, source) = {
        let mut conn = daemon.store.acquire().await?;
        let owner_agent = match tool.owner_agent {
            Some(id) => agents::by_id(&mut conn, id, false).await?,
            None => None,
        };
        let source = if tool.declared.secrets.is_empty() {
            None
        } else {
            match tool.scope {
                Scope::Instance => owner_agent.clone(),
                Scope::Global => match attachments::latest(&mut conn, tool.id).await? {
                    Some(attachment) => agents::by_id(&mut conn, attachment.agent, false).await?,
                    None => None,
                },
            }
        };
        (owner_agent, source)
    };
    let Some(owner) = tool.owner(owner_agent.as_ref()) else {
        return Err(StartError::Run("the tool's agent is gone".to_string()));
    };
    let binding = match source {
        Some(source) => secrets::bind(&tool.declared.secrets, &source.secrets).map_err(StartError::Unbound)?,
        None if tool.declared.secrets.is_empty() => IndexMap::new(),
        None => return Err(StartError::Run("no agent attached to the tool spawns it".to_string())),
    };
    let sender = Creator::Tool(creator::Tool::Dependency {
        template: tool.template.clone(),
        owner: owner.clone(),
    });
    let limits = tool.limits().map_err(StartError::Document)?;
    let needs = Needs {
        image: Some(tool.declared.image.clone()),
        memory: limits.0,
        disk: limits.1,
        ..Needs::default()
    };
    let candidates = provider::candidates(daemon, None, &needs).await.map_err(StartError::Provider)?;
    let container = build::alone(&tool.template, &tool.declared, limits);
    let attestation = attestation::Tool::Dependency {
        image: container.image.clone(),
        template: tool.template.clone(),
        owner,
    };
    let touched = watch::channel(Instant::now()).0;
    let mounts = Arc::new(Mounts::empty());
    let mut last = String::new();
    for (identity, handle) in candidates {
        let answerer = Arc::new(Answerer {
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
            inflight: Arc::new(Inflight::new()),
            files: Arc::new(Inflight::new()),
            kept: Mutex::new(None),
            declared: tool.declared.secrets.clone(),
            binding: binding.clone(),
            secrets: Mutex::new(None),
        });
        let started = tools_run::execute(&handle, &tools_request::Frame(container.clone()), answerer.set()).await;
        // The run is answered: its pull is over either way.
        answerer.pulls.release().await;
        let (id, handle, stream) = match started {
            Ok(opened) => opened,
            Err(error) => {
                secrets::abandon(daemon, &answerer).await;
                last = format!("{error:?}");
                continue;
            }
        };
        let permissions = answerer.permissions.lock().await.clone();
        let run = Arc::new(ToolRun {
            id: key,
            attestation: attestation.clone(),
            sender: sender.clone(),
            provider: identity,
            container: Some(id.id),
            handle: ToolHandle::Run(handle),
            eager: tool.is_eager(),
            connections: Mutex::new(HashSet::new()),
            held: watch::channel(false).0,
            inflight: Arc::new(Inflight::new()),
            files: Arc::new(Inflight::new()),
            touched: touched.clone(),
            mounts: Arc::clone(&mounts),
            volumes: Vec::new(),
            ended: watch::channel(false).0,
            tasks: Mutex::new(Vec::new()),
            edits: Mutex::new(()),
            dependency: Some(tool.clone()),
            sessions: Mutex::new(HashMap::new()),
            secrets: Mutex::new(answerer.secrets.lock().await.take()),
            epoch: daemon.live.epoch().await,
            log: daemon.live.tool_log(&daemon.store, key).await,
            ending: Mutex::new(None),
        });
        // From here a failure is the run's undoing.
        if let Err(error) = permissions::record(daemon, ToolRef::Dependency(tool.id), &permissions).await {
            undo_tool(daemon, run, false).await;
            return Err(error.into());
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
                // Rule 5.
                if !run.eager {
                    tasks.push(tokio::spawn(idle::idle_tool(Arc::clone(daemon), Arc::clone(&run))).abort_handle());
                }
            }
        }
        return Ok(run);
    }
    Err(StartError::Run(last))
}
