//! One run of a dependency tool.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use diverge_sdk::daemon::agent_log::Identity;
use diverge_sdk::daemon::creator::{self, Creator};
use diverge_sdk::daemon::lifecycle::Cause;
use diverge_sdk::provider::endpoints::containers::tools::run::client::{execute as tools_run, request as tools_request};
use diverge_sdk::shared::mcp::attestation;
use tokio::sync::{Mutex, watch};

use crate::containers::answerers::Answerer;
use crate::containers::capable::Needs;
use crate::containers::fuse::Mounts;
use crate::containers::images::Pulls;
use crate::containers::mcp::Served;
use crate::containers::start::Starting;
use crate::containers::{AgentRun, Inflight, Key, StartError, ToolHandle, ToolKey, ToolRun, build, idle, permissions, provider, pump_tool, undo_tool};
use crate::daemon::{Daemon, Kind};
use crate::judge::ToolRef;
use crate::store::agents;
use crate::store::dependency_tools::DependencyTool;

/// The agent's container a dependency tool mounts from: what a tool
/// that mounts its agent's files is deployed against. An agent's run
/// once it is live — [`Mounted::of`] — or, at the agent's
/// registration, when no run is made yet, what its answerer holds.
#[derive(Debug, Clone)]
pub struct Mounted {
    /// The provider the agent's container runs on: where its paths
    /// are served from.
    pub provider: Identity,
    /// The agent's container's id there.
    pub container: String,
    /// The agent's uses in progress: the tool's run is one, for its
    /// own life, so the agent's container is up for as long as the
    /// tool's is.
    pub files: Arc<Inflight>,
    /// Where a use of the agent's container is noted: at the end of
    /// the tool's use of it.
    pub touched: watch::Sender<Instant>,
}

impl Mounted {
    /// The agent's run, as a tool mounts from it.
    pub fn of(run: &AgentRun) -> Mounted {
        Mounted {
            provider: run.provider.clone(),
            container: run.container.clone(),
            files: Arc::clone(&run.files),
            touched: run.touched.clone(),
        }
    }
}

/// The dependency tool's run, deployed now if it was not up, `cause`
/// being what deployed it: any dependency tool's — lazy or eager, of
/// either scope.
///
/// The rules:
///
/// 1. One start at a time per tool: a second, meanwhile, waits and
///    finds the run the first made. A run live under the row is the
///    answer, and nothing is started.
/// 2. The container is the template's image, limits and arguments. A
///    tool that mounts its agent's files — only a lazy tool of the
///    `instance` scope may — is deployed against `mounted`, the
///    agent's container, its mounts the agent's paths served live
///    from there; a mount that cannot be served is the start's
///    failure. Every other tool has no mount, and `mounted` is not
///    read.
/// 3. It runs on the first candidate provider that will, under the
///    grants its program declared at its registration, kept on its
///    row, its run entered as live under its row with its pump.
/// 4. Its run has the idle clock every tool's run has —
///    [`idle_tool`](crate::containers::idle::idle_tool) — whatever its
///    mode and scope.
/// 5. A tool that mounts its agent's files holds a use on its agent's
///    run from the moment its run is made until it is down, so the
///    agent's container is up for as long as the tool's is.
/// 6. A per-agent tool whose agent is gone cannot be named and is not
///    started. A failure once the run is made undoes it as
///    [`undo_tool`](crate::containers::undo_tool) states.
///
/// The start runs in a task of its own, waited for: a caller
/// cancelled midway neither tears it nor orphans what it opened.
pub async fn tool(daemon: &Arc<Daemon>, tool: &DependencyTool, mounted: Option<Mounted>, cause: Cause) -> Result<Arc<ToolRun>, StartError> {
    let (daemon, tool) = (Arc::clone(daemon), tool.clone());
    let started = tokio::spawn(async move {
        // Rule 1.
        let _starting = Starting::hold(&daemon, Key::Tool(ToolKey::Dependency(tool.id))).await;
        start(&daemon, &tool, mounted, cause).await
    });
    started.await.unwrap_or_else(|error| Err(StartError::Unfinished(error.to_string())))
}

async fn start(daemon: &Arc<Daemon>, tool: &DependencyTool, mounted: Option<Mounted>, cause: Cause) -> Result<Arc<ToolRun>, StartError> {
    let key = ToolKey::Dependency(tool.id);
    if let Some(run) = daemon.live.tool_run(key).await {
        return Ok(run);
    }
    let owner_agent = {
        let mut conn = daemon.store.acquire().await?;
        match tool.owner_agent {
            Some(id) => agents::by_id(&mut conn, id, false).await?,
            None => None,
        }
    };
    let Some(owner) = tool.owner(owner_agent.as_ref()) else {
        return Err(StartError::Gone);
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
    // Rule 2.
    let mounted = if tool.declared.mounts() {
        let Some(mounted) = mounted else {
            return Err(StartError::Mounts("the tool mounts its agent's files, and its agent's container is not up".to_string()));
        };
        Some(mounted)
    } else {
        None
    };
    let mounts = Arc::new(match &mounted {
        Some(agent) => Mounts::build_dependency(daemon, &agent.provider, &agent.container, &tool.declared.fuse_file_mounts, &tool.declared.fuse_directory_mounts)
            .await
            .map_err(StartError::mounts)?,
        None => Mounts::empty(),
    });
    let container = match &mounted {
        Some(_) => build::dependency(&tool.template, &tool.declared, limits, &mounts).await,
        None => build::alone(&tool.template, &tool.declared, limits),
    };
    let attestation = attestation::Tool::Dependency {
        image: container.image.clone(),
        template: tool.template.clone(),
        owner,
    };
    let touched = watch::channel(Instant::now()).0;
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
            scopes: Arc::new(Inflight::new()),
            kept: Mutex::new(None),
        });
        let started = tools_run::execute(&handle, &tools_request::Frame(container.clone()), answerer.set()).await;
        // The run is answered: its pull is over either way.
        answerer.pulls.release().await;
        let (id, handle, stream) = match started {
            Ok(opened) => opened,
            Err(error) => {
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
            holds: daemon.live.holds(key).await,
            scopes: Arc::clone(&answerer.scopes),
            // Rule 5.
            agent: Mutex::new(mounted.as_ref().map(|agent| agent.files.enter_use(agent.touched.clone()))),
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
            epoch: daemon.live.epoch().await,
            log: daemon.live.tool_log(&daemon.store, key).await,
            ending: Mutex::new(None),
        });
        // From here a failure is the run's undoing.
        if let Err(error) = permissions::record(daemon, ToolRef::Dependency(tool.id), &permissions).await {
            undo_tool(daemon, run, false).await;
            return Err(error.into());
        }
        crate::containers::lifecycle::deployed(daemon.logs.clone(), run.id, Arc::clone(&run.log), run.epoch, run.provider.clone(), cause).await;
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
                // Rule 4.
                tasks.push(tokio::spawn(idle::idle_tool(Arc::clone(daemon), Arc::clone(&run))).abort_handle());
            }
        }
        return Ok(run);
    }
    mounts.stop().await;
    Err(StartError::Run(last))
}
