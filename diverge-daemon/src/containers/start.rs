//! Starting a run: the container made from the record.

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;

use chrono::Utc;
use diverge_sdk::shared::mcp::attestation;
use diverge_sdk::shared::relay::Relay;
use diverge_sdk::daemon::agent_log::{self as log, Item};
use diverge_sdk::daemon::lifecycle::Cause;
use diverge_sdk::daemon::endpoints::standalone_tools::connect::client::{execute as tools_connect, request as connect_request};
use diverge_sdk::shared::containers::info::Mode;
use diverge_sdk::provider::endpoints::containers::agents::run::client::{execute as agents_run, request as agents_request};
use diverge_sdk::provider::endpoints::containers::tools::run::client::{execute as tools_run, request as tools_request};
use tokio::sync::{Mutex, watch};

use super::answerers::Answerer;
use super::images::{self, Pulls};
use super::capable::Needs;
use super::fuse::Mounts;
use super::lifecycle::{self, End};
use super::mcp::Served;
use super::{AgentRun, Gate, Holding, Inflight, activity, deploy, Key, StartError, ToolHandle, ToolKey, ToolRun, build, idle, lost, provider, pump, pump_tool, sender_of_agent, sender_of_connected, sender_of_tool, serve_name, serve_name_connected, eager, stop};
use crate::daemon::{Daemon, Kind};
use crate::daemons;
use crate::containers::permissions;
use crate::judge::{Container, ToolRef};
use crate::store::agents::Agent;
use crate::store::connected_standalone_tools::ConnectedTool;
use crate::store::tools::{Tool, attachments};
use crate::store::{agents, agents_templates, connected_standalone_tools as connected_records, tools as tool_records, tools_templates};

/// The agent's run, started now if it was not up, `cause` being what
/// started it: the providers it
/// may run on found, the mounts served, the attached tools served,
/// the run opened on the first provider that runs it — its
/// dependencies deployed meanwhile, those that mount its files stopped
/// again when that provider does not — its dependencies settled to none
/// when its program declared none, its id taken, every tool it is
/// served deployed — see [`start_agent`]'s setup rules — the log told
/// it began, the pump and the idle clock running. A start that fails
/// keeps an `Error` item in the log, and the mounts opened are let go;
/// one that fails after
/// its run was made is undone as [`undo_agent`](stop::undo_agent)
/// states. The start runs in a task of its own, waited for: a caller
/// cancelled midway — an exchange aborted with its scope — neither
/// tears it nor orphans what it opened.
pub async fn agent(daemon: &Arc<Daemon>, agent: &Agent, cause: Cause) -> Result<Arc<AgentRun>, StartError> {
    let (daemon, agent) = (Arc::clone(daemon), agent.clone());
    let started = tokio::spawn(async move {
        let _starting = Starting::hold(&daemon, Key::Agent(agent.id)).await;
        if let Some(run) = daemon.live.agent_run(agent.id).await {
            return Ok(run);
        }
        match start_agent(&daemon, &agent, cause).await {
            Ok(run) => Ok(run),
            Err(error) => {
                let log = daemon.live.log(&daemon.store, agent.id).await;
                let item = Item::Error(log::Error {
                    r#type: Default::default(),
                    error: diverge_sdk::shared::error::Error(serde_json::json!({
                        "kind": "start",
                        "error": error.to_string(),
                    })),
                });
                let _ = lifecycle::note(daemon.logs.clone(), agent.id, log, item).await;
                daemon.live.changed(Kind::Agents);
                Err(error)
            }
        }
    });
    started.await.unwrap_or_else(|error| Err(StartError::Unfinished(error.to_string())))
}

/// [`agent`], as a future whose type is named: what a dependency
/// tool's deploy starts its agent by. An agent's start deploys the
/// tools its program declares, and the deploy of a tool that mounts
/// its agent's files starts its agent, so the two futures would each
/// hold the other; naming this one's type here is what lets the
/// compiler close the two.
pub fn agent_for(daemon: Arc<Daemon>, record: Agent, cause: Cause) -> Pin<Box<dyn Future<Output = Result<Arc<AgentRun>, StartError>> + Send>> {
    Box::pin(async move { agent(&daemon, &record, cause).await })
}

/// The start lock of one agent or tool, held for the length of its
/// start (see [`starting`](crate::daemon::Live::starting)): let go
/// however the start ends — returned, failed, panicked — and the
/// lock's entry then forgotten once nobody else holds it, and a
/// tool's holds with it where the start left no run and they hold
/// nothing. A drop
/// cannot wait on the live map's lock, so that is a task of its own.
pub(super) struct Starting {
    /// The daemon whose map the entry is in.
    daemon: Arc<Daemon>,
    /// Whose start it is.
    key: Key,
    /// The lock, held; taken at the drop, so the entry is unshared
    /// before it is looked at.
    held: Option<tokio::sync::OwnedMutexGuard<()>>,
}

impl Starting {
    /// The start lock of `key`, waited for and held.
    pub(super) async fn hold(daemon: &Arc<Daemon>, key: Key) -> Starting {
        let lock = daemon.live.starting(key).await;
        let held = lock.lock_owned().await;
        Starting { daemon: Arc::clone(daemon), key, held: Some(held) }
    }
}

impl Drop for Starting {
    fn drop(&mut self) {
        drop(self.held.take());
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            let (daemon, key) = (Arc::clone(&self.daemon), self.key);
            runtime.spawn(async move {
                daemon.live.started(key).await;
                if let Key::Tool(tool) = key {
                    daemon.live.release_holds(tool).await;
                }
            });
        }
    }
}

/// One start of an agent. Its SETUP, the part of it the agent's tools
/// are held through, by these rules:
///
/// 1. **The setup's hold begins at the registration.** Where the
///    program declared a dependency, the registration begins it, once
///    the declaration is on record — see
///    [`deploy`](super::deploy::deploy). Where it declared nothing it
///    is registered for nothing, and the hold begins as soon as the
///    container's id is out. From there the agent holds every tool it
///    is served — [`Holding`]. Tools the program declared on an
///    earlier start and not on this one are not served, not held and
///    not touched.
/// 2. **Every served tool is deployed, at once, and waited for** —
///    [`deploy::served`]: the dependency tools of either scope, lazy
///    or eager, the attached standalone tools and the connected ones.
///    This is before the run's `deployed` is kept: an agent whose
///    tools are not all up did not come up.
/// 3. **One that cannot be deployed fails the start**, naming it —
///    [`StartError::Tool`] — and so fails whatever asked for the
///    start: the message, the file operation, the connect. The run is
///    undone, the hold let go, and what did come up is left to its own
///    idle clock; no other provider is tried, since the tool's
///    failure is not this provider's.
/// 4. **The hold ends with the start**, unless the start was made for
///    a message: then the setup's count is kept on the run —
///    `arrival` — until the agent's outbox pump has entered the
///    message's own hold, so the tools are held from the registration
///    to the loop with no moment between. A start for anything else
///    lets the tools go as it ends, and their idle countdowns begin.
/// 5. **An attempt that fails lets go.** A provider that does not run
///    the container ends that attempt's hold; the next attempt's
///    registration begins its own.
async fn start_agent(daemon: &Arc<Daemon>, agent: &Agent, cause: Cause) -> Result<Arc<AgentRun>, StartError> {
    let (template, attached, connected) = {
        let mut conn = daemon.store.acquire().await?;
        let template = agents_templates::by_id(&mut conn, &agent.template, false)
            .await?
            .ok_or_else(|| StartError::NoTemplate(agent.template.clone()))?;
        // The agent's own attachments first, in attach order, then its
        // template's, so the prefixes are the same at every run; the
        // standalone tools before the connected ones, each family in
        // its own order.
        let mut ids = attachments::of_agent(&mut conn, agent.id).await?;
        for id in tool_records::template_attachments::of_template(&mut conn, &agent.template).await? {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
        let mut attached = Vec::new();
        for id in ids {
            if let Some(tool) = tool_records::by_id(&mut conn, id, false).await? {
                attached.push(tool);
            }
        }
        let mut ids = connected_records::attachments::of_agent(&mut conn, agent.id).await?;
        for id in connected_records::template_attachments::of_template(&mut conn, &agent.template).await? {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
        let mut connected = Vec::new();
        for id in ids {
            if let Some(tool) = connected_records::by_id(&mut conn, id, false).await? {
                connected.push(tool);
            }
        }
        (template, attached, connected)
    };
    let volumes = agent.provider.as_ref().map(|provider| provider.volume_mounts.as_slice()).unwrap_or(&[]);
    // What the container needs is the image's own to say: its
    // document, and what the template adds to it.
    let document = images::document(daemon, &build::image(&template.template.image)).await?.map_err(StartError::Document)?;
    let limits = images::limits(&document, template.template.extra_memory, template.template.extra_disk).map_err(StartError::Document)?;
    let needs = Needs {
        volume_modes: volumes.iter().map(|mount| mount.volume_mode).collect(),
        image: Some(build::image(&template.template.image)),
        memory: limits.0,
        disk: super::capable::overlay(limits.1, volumes),
    };
    let candidates = provider::candidates(daemon, agent.provider.as_ref().map(|provider| &provider.identity), &needs)
        .await
        .map_err(StartError::Provider)?;
    let mounts = Arc::new(
        Mounts::build(daemon, &agent.fuse_file_mounts, &agent.fuse_directory_mounts)
            .await
            .map_err(StartError::mounts)?,
    );
    let container = build::container(&agent.template, agent.index, &template.template, limits, volumes, &mounts).await;
    let image = container.image.clone();
    let served = Arc::new(Mutex::new(Served::new()));
    {
        let mut set = served.lock().await;
        for tool in attached {
            let name = serve_name(&tool);
            set.insert(tool, &name);
        }
        for tool in connected {
            let name = serve_name_connected(&tool);
            set.insert_connected(tool, &name);
        }
    }
    let touched = watch::channel(Instant::now()).0;
    let inflight = Arc::new(Inflight::new());
    let files = Arc::new(Inflight::new());
    let scopes = Arc::new(Inflight::new());
    let holding = Holding::new();
    let mut last = String::new();
    for (identity, handle) in candidates {
        let answerer = Arc::new(Answerer {
            daemon: Arc::clone(daemon),
            key: Key::Agent(agent.id),
            caller: Some(attestation::Caller::Agent(attestation::Agent {
                image: image.clone(),
                key: agent.key(),
            })),
            sender: sender_of_agent(agent),
            container: Container::Agent(agent.id),
            permissions: Mutex::new(Vec::new()),
            provider: identity.clone(),
            mounts: Arc::clone(&mounts),
            served: Arc::clone(&served),
            pulls: Arc::new(Pulls::new()),
            touched: touched.clone(),
            inflight: Arc::clone(&inflight),
            files: Arc::clone(&files),
            scopes: Arc::clone(&scopes),
            holding: Arc::clone(&holding),
            kept: Mutex::new(None),
        });
        let opened = agents_run::execute(&handle, &agents_request::Frame(container.clone()), answerer.set()).await;
        // The run is answered: its pull is over either way.
        answerer.pulls.release().await;
        let (id, handle, stream) = match opened {
            Ok(opened) => opened,
            Err(error) => {
                // The dependencies served for this attempt go with it,
                // and those that mount this attempt's container are
                // stopped: the next attempt registers anew.
                let own = {
                    let mut set = served.lock().await;
                    let own = set.own();
                    for id in &own {
                        set.remove(ToolKey::Dependency(*id));
                    }
                    own
                };
                stop::stop_mounted(daemon, &own, End::Expected).await;
                // Setup rule 5.
                holding.forget(daemon).await;
                last = format!("{error:?}");
                continue;
            }
        };
        if answerer.kept.lock().await.is_none() {
            eager::settle(daemon, agent.id).await;
        }
        // Setup rule 1: begun by the registration where there was
        // one, and here where the program declared nothing.
        holding.begin(daemon, &served, &[]).await;
        let run = Arc::new(AgentRun {
            id: agent.id,
            attestation: attestation::Caller::Agent(attestation::Agent {
                image,
                key: agent.key(),
            }),
            sender: sender_of_agent(agent),
            provider: identity.clone(),
            container: id.id,
            handle,
            loop_active: watch::channel(false).0,
            inactives: watch::channel(0).0,
            inflight,
            files,
            scopes,
            touched,
            mounts,
            volumes: crate::volumes::of_agent(agent),
            served,
            holding: Arc::clone(&holding),
            arrival: Mutex::new(None),
            messages: Arc::new(Mutex::new(HashMap::new())),
            pending: Arc::new(Inflight::new()),
            gate: Mutex::new(Gate { ending: false }),
            tasks: Mutex::new(Vec::new()),
            edits: Mutex::new(()),
            epoch: daemon.live.epoch().await,
            index: Mutex::new(0),
            decisions: Mutex::new(None),
            log: daemon.live.log(&daemon.store, agent.id).await,
            ending: Mutex::new(None),
            ended: watch::channel(false).0,
        });
        // Setup rules 2 and 3.
        if let Err((name, reason)) = deploy::served(daemon, &run.served, &deploy::Mounted::of(&run), deploy::Causes::SETUP).await {
            stop::undo_agent(daemon, run, false).await;
            return Err(StartError::Tool { name, reason });
        }
        lifecycle::deployed(daemon.logs.clone(), run.id, Arc::clone(&run.log), run.epoch, identity.clone(), cause).await;
        // From here a failure is the run's undoing: its container, its
        // dependencies, its mounts and its `deployed`. What its program
        // declared it may ask of the daemon is kept on its row.
        let permissions = answerer.permissions.lock().await.clone();
        let recorded: Result<(), crate::store::Error> = async {
            let mut conn = daemon.store.acquire().await?;
            agents::set_permissions(&mut conn, agent.id, &permissions).await?;
            agents::set_deployed(&mut conn, agent.id, &identity, Utc::now()).await?;
            Ok(())
        }
        .await;
        if let Err(error) = recorded {
            stop::undo_agent(daemon, run, false).await;
            return Err(error.into());
        }
        daemon.live.insert_agent(Arc::clone(&run)).await;
        daemon.live.changed(Kind::Agents);
        daemon.live.nudge_capacity(&run.provider);
        // The tasks are spawned and kept under their lock, taken first,
        // and only while the run is still the live one: an end that
        // comes meanwhile drains them after, and one that came already
        // drained none of these.
        {
            let mut tasks = run.tasks.lock().await;
            if daemon.live.is_agent_run(&run).await {
                let pumping = tokio::spawn(pump::pump(Arc::clone(daemon), Arc::clone(&run), stream));
                let clock = tokio::spawn(idle::idle(Arc::clone(daemon), Arc::clone(&run)));
                let watching = tokio::spawn(activity::activity(Arc::clone(daemon), Arc::clone(&run)));
                // A mount's serve lost is the run's end: see `lost_agent`.
                let losing = tokio::spawn(lost::lost_agent(Arc::clone(&run)));
                tasks.extend([pumping.abort_handle(), clock.abort_handle(), watching.abort_handle(), losing.abort_handle()]);
            }
        }
        // Setup rule 4.
        let setup = run.holding.end().await;
        if cause == Cause::Message {
            *run.arrival.lock().await = setup;
        }
        return Ok(run);
    }
    mounts.stop().await;
    Err(StartError::Run(last))
}

/// The tool's run, started now, `cause` being what started it: its
/// container run on the first of its
/// providers that runs it, with its mounts served.
/// [`pump_tool`](super::pump_tool) reads the
/// container's activity to its end and takes the run down; its idle
/// clock ends it `idle_seconds` after the last hold left it, as
/// [`idle_tool`](super::idle_tool) states. A tool declares no
/// dependencies, so nothing is served to it. Whether the tool is a
/// eager tool is its image's mode, read here for the run. A start that
/// fails after its run was made is undone as
/// [`undo_tool`](stop::undo_tool) states; the start runs in a task of
/// its own, waited for, as an agent's does. A connected tool's run is
/// [`connected`]'s.
pub async fn tool(daemon: &Arc<Daemon>, tool: &Tool, cause: Cause) -> Result<Arc<ToolRun>, StartError> {
    let (daemon, tool) = (Arc::clone(daemon), tool.clone());
    let started = tokio::spawn(async move {
        let _starting = Starting::hold(&daemon, Key::Tool(ToolKey::Record(tool.id))).await;
        if let Some(run) = daemon.live.tool_run(ToolKey::Record(tool.id)).await {
            return Ok(run);
        }
        start_tool(&daemon, &tool, cause).await
    });
    started.await.unwrap_or_else(|error| Err(StartError::Unfinished(error.to_string())))
}

async fn start_tool(daemon: &Arc<Daemon>, tool: &Tool, cause: Cause) -> Result<Arc<ToolRun>, StartError> {
    let touched = watch::channel(Instant::now()).0;
    let (template, pinned) = (&tool.template, &tool.provider);
    let (identity, container, handle, mounts, attestation, eager, stream, permissions, scopes) = {
        let record = {
            let mut conn = daemon.store.acquire().await?;
            tools_templates::by_id(&mut conn, template, false)
                .await?
                .ok_or_else(|| StartError::NoTemplate(template.clone()))?
        };
        let volumes = pinned.as_ref().map(|provider| provider.volume_mounts.as_slice()).unwrap_or(&[]);
        // What the container needs, and how it is kept, is the
        // image's own to say: its document, and what the template
        // adds to it.
        let document = images::document(daemon, &build::image(&record.template.image)).await?.map_err(StartError::Document)?;
        let limits = images::limits(&document, record.template.extra_memory, record.template.extra_disk).map_err(StartError::Document)?;
        let needs = Needs {
            volume_modes: volumes.iter().map(|mount| mount.volume_mode).collect(),
            image: Some(build::image(&record.template.image)),
            memory: limits.0,
            disk: super::capable::overlay(limits.1, volumes),
        };
        let candidates = provider::candidates(daemon, pinned.as_ref().map(|provider| &provider.identity), &needs)
            .await
            .map_err(StartError::Provider)?;
        let mounts = Arc::new(
            Mounts::build(daemon, &tool.fuse_file_mounts, &tool.fuse_directory_mounts)
                .await
                .map_err(StartError::mounts)?,
        );
        let container = build::container(template, tool.index, &record.template, limits, volumes, &mounts).await;
        let image = container.image.clone();
        let mut last = String::new();
        let mut opened = None;
        for (identity, handle) in candidates {
            let answerer = Arc::new(Answerer {
                daemon: Arc::clone(daemon),
                key: Key::Tool(ToolKey::Record(tool.id)),
                caller: None,
                sender: sender_of_tool(tool),
                container: Container::Tool(ToolRef::Standalone(tool.id)),
                permissions: Mutex::new(Vec::new()),
                provider: identity.clone(),
                mounts: Arc::clone(&mounts),
                served: Arc::new(Mutex::new(Served::new())),
                pulls: Arc::new(Pulls::new()),
                touched: touched.clone(),
                inflight: Arc::new(Inflight::new()),
                files: Arc::new(Inflight::new()),
                scopes: Arc::new(Inflight::new()),
                holding: Holding::new(),
                kept: Mutex::new(None),
            });
            let started = tools_run::execute(&handle, &tools_request::Frame(container.clone()), answerer.set()).await;
            // The run is answered: its pull is over either way.
            answerer.pulls.release().await;
            match started {
                Ok((id, handle, stream)) => {
                    let permissions = answerer.permissions.lock().await.clone();
                    opened = Some((identity, id.id, handle, stream, permissions, Arc::clone(&answerer.scopes)));
                    break;
                }
                Err(error) => {
                    last = format!("{error:?}");
                }
            }
        }
        let Some((identity, id, handle, stream, permissions, scopes)) = opened else {
            mounts.stop().await;
            return Err(StartError::Run(last));
        };
        let attestation = attestation::Tool::Created {
            image,
            template: template.clone(),
            index: tool.index,
            name: tool.name.clone(),
        };
        let eager = document.mode() == Some(Mode::Eager);
        (identity, Some(id), ToolHandle::Run(handle), mounts, attestation, eager, stream, permissions, scopes)
    };
    let run = Arc::new(ToolRun {
        id: ToolKey::Record(tool.id),
        attestation,
        sender: sender_of_tool(tool),
        provider: identity.clone(),
        container,
        handle,
        eager,
        connections: Mutex::new(HashSet::new()),
        held: watch::channel(false).0,
        holds: daemon.live.holds(ToolKey::Record(tool.id)).await,
        scopes,
        agent: Mutex::new(None),
        inflight: Arc::new(Inflight::new()),
        files: Arc::new(Inflight::new()),
        touched,
        mounts,
        volumes: crate::volumes::of_tool(tool),
        ended: watch::channel(false).0,
        tasks: Mutex::new(Vec::new()),
        edits: Mutex::new(()),
        dependency: None,
        sessions: Mutex::new(HashMap::new()),
        epoch: daemon.live.epoch().await,
        log: daemon.live.tool_log(&daemon.store, ToolKey::Record(tool.id)).await,
        ending: Mutex::new(None),
    });
    // From here a failure is the run's undoing: its container, its
    // mounts and its `deployed`. What its program
    // declared it may ask of the daemon is kept on its row.
    let recorded = permissions::record(daemon, Container::Tool(ToolRef::Standalone(tool.id)), &permissions).await;
    if let Err(error) = recorded {
        stop::undo_tool(daemon, run, false).await;
        return Err(error.into());
    }
    lifecycle::deployed(daemon.logs.clone(), run.id, Arc::clone(&run.log), run.epoch, identity.clone(), cause).await;
    let recorded: Result<(), crate::store::Error> = async {
        let mut conn = daemon.store.acquire().await?;
        tool_records::set_deployed(&mut conn, tool.id, &identity, Utc::now()).await?;
        Ok(())
    }
    .await;
    if let Err(error) = recorded {
        stop::undo_tool(daemon, run, false).await;
        return Err(error.into());
    }
    daemon.live.insert_tool(Arc::clone(&run)).await;
    daemon.live.changed(Kind::StandaloneTools);
    daemon.live.nudge_capacity(&run.provider);
    // Spawned and kept under the tasks' lock, while the run is still
    // the live one, as an agent's are.
    {
        let mut tasks = run.tasks.lock().await;
        if daemon.live.is_tool_run(&run).await {
            tasks.push(tokio::spawn(pump_tool::pump_tool(Arc::clone(daemon), Arc::clone(&run), stream)).abort_handle());
            // A mount's serve lost is the run's end: see `lost_tool`.
            tasks.push(tokio::spawn(lost::lost_tool(Arc::clone(&run))).abort_handle());
            tasks.push(tokio::spawn(idle::idle_tool(Arc::clone(daemon), Arc::clone(&run))).abort_handle());
        }
    }
    Ok(run)
}

/// The connected tool's run, connected now, `cause` being what
/// connected it: the daemon its record
/// names connected to, over the one connection to it, and that
/// daemon's connect opened naming the tool, relayed along the record's
/// route when it has one, which starts the container on the daemon
/// that runs it and carries the tool's MCP exchanges from then on.
/// Nothing of the container is this daemon's: no mount, no
/// provider reserved, no permission kept. The waiter takes the run down
/// when the connect scope ends, which the other daemon finishes when
/// the tool's run ends there; the idle clock ends it `idle_seconds`
/// after the last hold left it. A connected tool is never a
/// eager tool here. The start runs in a task of its own, waited for, as a
/// tool's does.
pub async fn connected(daemon: &Arc<Daemon>, tool: &ConnectedTool, cause: Cause) -> Result<Arc<ToolRun>, StartError> {
    let (daemon, tool) = (Arc::clone(daemon), tool.clone());
    let started = tokio::spawn(async move {
        let _starting = Starting::hold(&daemon, Key::Tool(ToolKey::Connected(tool.id))).await;
        if let Some(run) = daemon.live.tool_run(ToolKey::Connected(tool.id)).await {
            return Ok(run);
        }
        start_connected(&daemon, &tool, cause).await
    });
    started.await.unwrap_or_else(|error| Err(StartError::Unfinished(error.to_string())))
}

async fn start_connected(daemon: &Arc<Daemon>, tool: &ConnectedTool, cause: Cause) -> Result<Arc<ToolRun>, StartError> {
    let record = {
        let mut conn = daemon.store.acquire().await?;
        crate::store::daemons::by_id(&mut conn, tool.daemon, false).await?
    };
    let record = record.ok_or(StartError::NoDaemon(tool.daemon))?;
    let peer = daemons::connect(daemon, &record).await.map_err(StartError::Daemon)?;
    // The connect relayed along the record's route, when it has one:
    // each hop served as an account, as any relay is.
    let request = connect_request::Frame {
        tool: tool.tool.clone(),
        relay: (!tool.route.is_empty()).then(|| Relay(tool.route.clone())),
    };
    let connected = tools_connect::execute(&peer.connected.handle, &request)
        .await
        .map_err(|error| StartError::Run(connect_failed(&error)))?;
    let attestation = attestation::Tool::Connected {
        image: connected.image().clone(),
        template: connected.template().to_string(),
        index: tool.index,
        name: tool.name.clone(),
        daemon: tool.daemon,
    };
    let identity = peer.provider.clone();
    let key = ToolKey::Connected(tool.id);
    let run = Arc::new(ToolRun {
        id: key,
        attestation,
        sender: sender_of_connected(tool),
        provider: identity.clone(),
        container: None,
        handle: ToolHandle::Connected(connected),
        eager: false,
        connections: Mutex::new(HashSet::new()),
        held: watch::channel(false).0,
        holds: daemon.live.holds(key).await,
        // A connected tool's program is another daemon's, and opens
        // nothing on this one.
        scopes: Arc::new(Inflight::new()),
        agent: Mutex::new(None),
        inflight: Arc::new(Inflight::new()),
        files: Arc::new(Inflight::new()),
        touched: watch::channel(Instant::now()).0,
        mounts: Arc::new(Mounts::empty()),
        volumes: Vec::new(),
        ended: watch::channel(false).0,
        tasks: Mutex::new(Vec::new()),
        edits: Mutex::new(()),
        dependency: None,
        sessions: Mutex::new(HashMap::new()),
        epoch: daemon.live.epoch().await,
        log: daemon.live.tool_log(&daemon.store, key).await,
        ending: Mutex::new(None),
    });
    lifecycle::deployed(daemon.logs.clone(), run.id, Arc::clone(&run.log), run.epoch, identity, cause).await;
    // From here a failure is the run's undoing: its connect and its
    // `deployed`.
    let recorded: Result<(), crate::store::Error> = async {
        let mut conn = daemon.store.acquire().await?;
        connected_records::set_deployed(&mut conn, tool.id, Utc::now()).await?;
        Ok(())
    }
    .await;
    if let Err(error) = recorded {
        stop::undo_tool(daemon, run, false).await;
        return Err(error.into());
    }
    daemon.live.insert_tool(Arc::clone(&run)).await;
    daemon.live.changed(Kind::ConnectedStandaloneTools);
    // Spawned and kept under the tasks' lock, while the run is still
    // the live one, as an agent's are.
    {
        let mut tasks = run.tasks.lock().await;
        if daemon.live.is_tool_run(&run).await {
            let waiter = {
                let daemon = Arc::clone(daemon);
                let run = Arc::clone(&run);
                tokio::spawn(async move {
                    let ended = run.handle.wait().await;
                    run.end_as(End::Unexpected(ended.reason())).await;
                    stop::end_tool(daemon, run);
                })
            };
            tasks.push(waiter.abort_handle());
            tasks.push(tokio::spawn(idle::idle_tool(Arc::clone(daemon), Arc::clone(&run))).abort_handle());
        }
    }
    Ok(run)
}

/// Why the other daemon's connect did not connect, in a start's words:
/// the daemon's own answers named, and this end's failures as they
/// say themselves.
fn connect_failed(error: &tools_connect::ExecuteError) -> String {
    match error {
        tools_connect::ExecuteError::NotFound => "the daemon named has no such tool".to_string(),
        tools_connect::ExecuteError::Forbidden => "the daemon named does not allow this daemon to connect to the tool".to_string(),
        tools_connect::ExecuteError::Daemon(error) => format!("the daemon named could not serve the tool: {}", error.0),
        error => format!("the connect on the daemon named failed: {error}"),
    }
}
