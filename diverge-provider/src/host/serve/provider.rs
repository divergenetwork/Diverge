//! What every connection is served with.

use std::collections::HashSet;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::pin::pin;
use std::sync::Arc;

use futures_util::future::{self, Either};
use tokio::sync::watch;

use diverge_sdk::shared::Identity;
use diverge_sdk::wire::connection::Connection;
use diverge_sdk::wire::handshake::{self, Handshook};
use crate::protocol::acceptors::Acceptors;
use crate::protocol::directory::Directory;
use crate::protocol::listables::Listables;
use crate::protocol::mailboxes::Mailboxes;
use crate::protocol::changes::Changes;
use crate::protocol::handle::handle;
use diverge_sdk::provider::endpoints::info::server::response::Capabilities;
use super::capabilities::{capabilities, registries};
use diverge_sdk::wire::server::session::Session;

use crate::config::Config;
use crate::config::images::Images;
use crate::config::volumes::Volumes;

use super::Error;
use crate::host::capacity_reporter::CapacityReporter;
use crate::host::container_deployer::ContainerDeployer;
use crate::host::image_manager::{ImageManager, Store};
use crate::host::image_registry::ImageRegistry;
use crate::host::registry_client::RegistryClient;
use crate::host::tools::podman;
use crate::host::authenticator::ProviderAuthenticator;
use crate::host::volume_manager::{self, Scratch, VolumeManager};
use crate::host::Limit;
use crate::log::Log;

/// The provider's pieces, built once and shared by every connection:
/// the protocol takes each behind an `Arc`, and the directory of running
/// containers and the acceptors of daemon connections are one per
/// provider by the protocol's own rule, since a connector on one
/// connection names a container run, or a daemon accepting, on
/// another.
#[derive(Debug)]
pub struct Provider {
    /// The provider's side of every handshake: its own tokens, and the
    /// identities it serves.
    pub(super) authenticator: Arc<ProviderAuthenticator>,
    /// The deployer, held to the end so its tunnel lives as long:
    /// `None` on a provider that uses no podman.
    pub(super) deployer: Arc<Option<Arc<ContainerDeployer>>>,
    /// The volumes.
    volumes: Arc<VolumeManager>,
    /// The Store's manager, which is the OCI store too: the image
    /// served to a caller's pull, held and proven from one
    /// resolution.
    images: Arc<ImageManager>,
    /// The running containers, one map for the provider.
    directory: Arc<Directory>,
    /// The daemons accepting connections from other daemons, one map
    /// for the provider.
    acceptors: Arc<Acceptors>,
    /// The connected identities that hold a `daemons::announce` open,
    /// one set for the provider.
    listables: Arc<Listables>,
    /// The identities listening for messages, what waits for each, and
    /// every sender's window, one for the provider.
    mailboxes: Arc<Mailboxes>,
    /// Every change to any caller's volumes, for the listings open.
    volume_changes: Arc<Changes>,
    /// Every change to any caller's images, for the listings open.
    image_changes: Arc<Changes>,
    /// What this provider holds out, answered by `info` and enforced
    /// by every handler: read off the configuration once.
    capabilities: Arc<Capabilities>,
    /// How much a caller may still use, answered by `capacity`: the
    /// stores and the deployer's caps, asked as of each answer.
    capacity: Arc<CapacityReporter>,
    /// The provider's log: see [`Log`].
    pub(super) log: Arc<Log>,
    /// The provider's stop: sent `true` by a loopback `POST /stop`
    /// (see [`stop`](super::stop)), or by the signal, and read by the
    /// listener, which drains on it, and by every connection, which
    /// closes on it. Made by [`run`](super::run) and held here so a
    /// handler can reach it.
    pub stop: watch::Sender<bool>,
}

impl Provider {
    /// Everything made, in the order its parts depend on each other:
    /// the provider's disk readied — `dir`, `<root>/provider/`, with
    /// `run/`, podman's `storage_path` and every store
    /// made, every fixed volume found to be a directory, every store's
    /// capacity and every fixed name checked; the Store, where the configuration names one — else
    /// the deployer's own under `run/`, with no room — made and its
    /// `tmp/` swept first of all, since the registry serves it; the
    /// container overlay cap and the scratch directory, swept; the
    /// volumes; then, on a provider that uses podman and on no other
    /// — see [`Config::podman_in_use`] — podman told where its data
    /// is, the registry started, since the deployer mints its tokens
    /// and holds it for its life, and the deployer, which
    /// brings the machine up, sweeps an earlier life away, writes the
    /// auth file and opens the tunnel; the capacity reporter, given the
    /// volumes, the store and the deployer; the registry client, ONE
    /// for the provider, which a push and a pull both fetch with;
    /// the image manager — the OCI store too — given the store, the blob store, the deployer and
    /// the registry client; and the directory. The store is handed to
    /// the registry, the deployer, the reporter and the manager, and
    /// the blob store to the registry, the deployer and the manager.
    /// `stop` is the caller's, made before anything here, so
    /// the signal and the stop route send on the one sender.
    /// `authenticator` is the provider's side of every handshake, made
    /// already.
    pub async fn start(config: Config, dir: PathBuf, stop: watch::Sender<bool>, authenticator: Arc<ProviderAuthenticator>) -> Result<Self, Error> {
        ready(&config, &dir).await?;
        let log = Arc::new(Log::start(&dir, config.logging).await);
        log.say("the provider started".to_string());
        let capabilities = Arc::new(capabilities(&config));
        let podman_in_use = config.podman_in_use();
        let registries = registries(&config).to_vec();
        let mut shares = config.volumes.as_ref().map(Volumes::paths).unwrap_or_default();
        let store = config.images.as_ref().map(|images| {
            shares.push(images.path.clone());
            Arc::new(Store::new(images))
        });
        // The Store the deployer and the registry keep every blob a
        // run takes from a caller in: the configured one, or — with no
        // `images` section — one of the deployer's own under `run/`,
        // with no room, which no identity owns anything in and whose
        // cache is trimmed as soon as it is written.
        let blobs = match &store {
            Some(store) => Arc::clone(store),
            None => Arc::new(Store::new(&Images {
                path: dir.join("run").join("images"),
                capacity: 0,
            })),
        };
        blobs.sweep().await.map_err(Error::Store)?;
        // The containers' overlay cap: their `disk`, and their
        // ephemeral mounts' `overlay_disk`. `0` where the provider
        // runs none.
        let disk = Arc::new(Limit::new(config.containers.as_ref().map_or(0, |containers| containers.container_overlay_disk)));
        // Where a container's ephemeral mount keeps its layer: under
        // podman's storage path, which a podman machine must see for
        // the layer to be mounted inside it. Only a provider that
        // runs containers has one.
        if config.containers.is_some() {
            shares.push(volume_manager::scratch_dir(&config.podman.storage_path));
        }
        // The FUSE serves' own: where an ephemeral serve keeps its
        // layer, and the cap those layers count against. With no
        // `volumes.ephemeral_fuse` there is neither, and no serve of
        // an ephemeral volume is taken: see `capabilities`.
        let ephemeral_fuse = config.volumes.as_ref().and_then(|volumes| volumes.ephemeral_fuse.clone());
        let fuse_disk = Arc::new(Limit::new(ephemeral_fuse.as_ref().map_or(0, |ephemeral_fuse| ephemeral_fuse.capacity)));
        let scratch = Scratch::new(ephemeral_fuse.map(|ephemeral_fuse| ephemeral_fuse.path), Arc::clone(&fuse_disk));
        scratch.sweep().await.map_err(Error::Scratch)?;
        let resizable = cfg!(target_os = "linux") || podman_in_use;
        let volumes = Arc::new(VolumeManager::new(config.volumes, scratch, resizable));
        // Podman, and everything that is there for it — the registry
        // it pulls a caller's image from, and the deployer — only on
        // a provider that uses it: see `Config::podman_in_use`. One
        // that does not starts no podman here or after.
        let deployer = if podman_in_use {
            podman::configure(config.podman.storage_path.clone());
            let registry = Arc::new(ImageRegistry::start(Arc::clone(&blobs)).await.map_err(Error::Registry)?);
            Some(Arc::new(
                ContainerDeployer::new(config.podman, config.containers, dir, Arc::clone(&volumes), registry, shares, disk, Arc::clone(&blobs))
                    .await
                    .map_err(Error::Deployer)?,
            ))
        } else {
            None
        };
        let capacity = Arc::new(CapacityReporter::new(Arc::clone(&volumes), store.clone(), deployer.clone(), Arc::clone(&fuse_disk)));
        let client = Arc::new(RegistryClient::new(&registries).map_err(Error::RegistryClient)?);
        let images = Arc::new(ImageManager::new(store, blobs, Arc::clone(&volumes), deployer.clone(), client));
        let deployer = Arc::new(deployer);
        let listables = Arc::new(Listables::new());
        Ok(Provider {
            authenticator,
            deployer,
            volumes,
            images,
            directory: Arc::new(Directory::new()),
            acceptors: Arc::new(Acceptors::new()),
            mailboxes: Arc::new(Mailboxes::new(capabilities.messages, Arc::clone(&listables))),
            listables,
            volume_changes: Arc::new(Changes::new()),
            image_changes: Arc::new(Changes::new()),
            capabilities,
            capacity,
            log,
            stop,
        })
    }

    /// A connection a peer dialled: its handshake as the connectee —
    /// see [`authenticator`](crate::host::authenticator) — and then the
    /// connection served under the peer's identity, which it holds for
    /// as long as it lasts. A handshake that fails — an identity
    /// another connection holds among the ways — closes the connection
    /// with nothing sent.
    pub async fn accept(&self, connection: Connection, address: IpAddr) {
        match handshake::accept(connection, &self.authenticator.admitting(), address).await {
            Ok(Handshook { connection, peer }) => self.held(connection, peer, address, None).await,
            Err(error) => self.log.say(format!("{address} dialled in, and its handshake was not taken: {error}")),
        }
    }

    /// A connection past its handshake, under `peer`, whose identity
    /// the handshake took: the connection served, and the identity let
    /// go. `address` is the socket's other end, and `dialled` the
    /// address this provider dialled it at, when it did: neither lists
    /// the peer, and both are kept for the connection's life, as what a
    /// `daemons::announce` the peer opens on it — whichever end dialled
    /// — is listed with, for as long as that scope is open.
    pub async fn held(&self, connection: Connection, peer: Identity, address: IpAddr, dialled: Option<String>) {
        let how = match &dialled {
            Some(target) => format!("dialled at {target}"),
            None => "dialled in".to_string(),
        };
        self.log.say(format!("{address} {how}, connected as {} {}", peer.issuer, peer.subject));
        let began = tokio::time::Instant::now();
        self.connection(connection, Arc::new(peer.clone()), address, dialled).await;
        self.mailboxes.left(&peer).await;
        self.authenticator.release(&peer).await;
        self.log.say(format!("{address} {} {} disconnected after {}s", peer.issuer, peer.subject, began.elapsed().as_secs()));
    }

    /// One connection, served for as long as it lasts, or until the
    /// provider stops: the protocol's session over the socket and its
    /// `handle` with everything here, raced against `stop` turning
    /// `true`. Why it ended is not kept — a peer that broke the
    /// handshake's rule and a peer that hung up are both a connection
    /// that is over, and there is nowhere to say which.
    ///
    /// # The stop closes the socket
    ///
    /// The listener's drain waits only for HTTP requests in flight;
    /// an upgraded socket lives in its own task, and a dialled one in
    /// its dial's, and neither is the drain's to close. So the stop is
    /// read here, by every connection, and when it wins the race the
    /// `handle` in flight is dropped — the session with it, which is
    /// how the protocol ends a connection: every scope's feeds close,
    /// and the socket goes with the session. What the drop also takes
    /// is the handle's set of scopes, so a scope still being served
    /// is aborted rather than drained; the sweep that follows every
    /// stop removes every container of this provider by force, and the
    /// provider is dropped after it, so nothing a scope would have
    /// torn down outlives the stop.
    async fn connection(&self, connection: Connection, client_identity: Arc<Identity>, address: IpAddr, dialled: Option<String>) {
        let session = Session::new(connection);
        let mut stop = self.stop.subscribe();
        let serving = pin!(handle(
            session,
            client_identity,
            address,
            dialled,
            Arc::clone(&self.deployer),
            Arc::clone(&self.volumes),
            Arc::clone(&self.images),
            Arc::clone(&self.images),
            Arc::clone(&self.capacity),
            Arc::clone(&self.directory),
            Arc::clone(&self.volume_changes),
            Arc::clone(&self.image_changes),
            Arc::clone(&self.acceptors),
            Arc::clone(&self.listables),
            Arc::clone(&self.mailboxes),
            Arc::clone(&self.capabilities),
            Arc::clone(&self.log),
        ));
        let stopping = pin!(async move {
            let _ = stop.wait_for(|stopped| *stopped).await;
        });
        match future::select(serving, stopping).await {
            Either::Left((_, _)) => {}
            Either::Right(((), serving)) => drop(serving),
        }
    }
}

/// The provider's disk, as its block names it: `dir` with `run/`
/// inside it, podman's `storage_path` — on a provider
/// that uses podman, and on no other — and every store made if
/// absent, the Store's directory made if absent and
/// `run/builds/` swept; a fixed
/// volume whose path is not an existing directory refused, since a
/// fixed volume is content the operator already has; a store whose
/// capacity is `0` refused, and the Store's likewise; a fixed
/// volume whose name is not one a volume may have, or is another
/// fixed volume's, refused. What the file said is the SDK's to read; what it names
/// here is the provider's to check.
async fn ready(config: &Config, dir: &Path) -> Result<(), Error> {
    let mut made = vec![dir.join("run")];
    if config.podman_in_use() {
        made.push(config.podman.storage_path.clone());
    }
    for made in made {
        tokio::fs::create_dir_all(&made).await.map_err(|source| Error::Directory(made, source))?;
    }
    let builds = dir.join("run").join("builds");
    let _ = tokio::fs::remove_dir_all(&builds).await;
    tokio::fs::create_dir_all(&builds).await.map_err(|source| Error::Directory(builds, source))?;
    if let Some(images) = &config.images {
        if images.capacity == 0 {
            return Err(Error::ImageCapacity(images.path.clone()));
        }
        tokio::fs::create_dir_all(&images.path)
            .await
            .map_err(|source| Error::Directory(images.path.clone(), source))?;
    }
    let Some(volumes) = &config.volumes else {
        return Ok(());
    };
    for store in volumes.stores.iter() {
        if store.capacity == 0 {
            return Err(Error::Capacity(store.path.clone()));
        }
        tokio::fs::create_dir_all(&store.path)
            .await
            .map_err(|source| Error::Directory(store.path.clone(), source))?;
    }
    // An ephemeral FUSE scratch with no room serves nothing, and is
    // refused as a store with none is; its directory is the scratch's
    // own to make, when it is swept.
    if let Some(ephemeral_fuse) = &volumes.ephemeral_fuse {
        if ephemeral_fuse.capacity == 0 {
            return Err(Error::Capacity(ephemeral_fuse.path.clone()));
        }
    }
    let mut names = HashSet::new();
    for fixed in volumes.fixed.iter() {
        if !tokio::fs::metadata(&fixed.path).await.is_ok_and(|found| found.is_dir()) {
            return Err(Error::FixedMissing(fixed.path.clone()));
        }
        if !volume_manager::ok(&fixed.name) || !names.insert(fixed.name.as_str()) {
            return Err(Error::FixedName(fixed.name.clone()));
        }
    }
    Ok(())
}
