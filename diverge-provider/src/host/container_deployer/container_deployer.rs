//! The deployer.

use diverge_sdk::shared::Identity;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::protocol::caller::Caller;
use crate::protocol::container_deployer::{self, Deployed, Refused};
use crate::protocol::deployment::Deployment;
use futures_util::future;

use super::deploy::deploy;
use super::info::read_file;
use crate::protocol::image_info::ImageInfo;
use super::mounts::tool_path;
use super::{Container, Error, Images, Shared, name_ok};
use crate::host::Limit;
use crate::host::image_manager::Store;
use crate::host::image_registry::ImageRegistry;
use crate::config::containers::Containers;
use crate::config::podman::Podman;
use crate::host::tools::{mount, podman};
use crate::host::volume_manager::VolumeManager;
use diverge_sdk::shared::containers::request::Image;

/// The provider's container deployer: podman, with the caps, the
/// image cache, the volumes, and the way to the registry.
///
/// A provider has one exactly where it uses podman — see
/// [`Config::podman_in_use`](crate::config::Config::podman_in_use) —
/// and what of it is there follows what the provider uses podman for:
///
/// - with a `containers` section or an `images` section: podman's
///   storage, the machine on the hosts with one, the auth file, the
///   mounts directory, the image cache, the registry and the tunnel
///   to it, and the sweep of an earlier life. A build and an
///   `images::read` need each, as a run does;
/// - with a `containers` section only: the proxy binary, which a
///   container is handed, and the `memory` cap. Without the section
///   there is no proxy path and the cap is `0`, and no run reaches
///   the deployer, every one having been answered `Unsupported`.
#[derive(Debug)]
pub struct ContainerDeployer {
    /// The `podman` section: the registries, and where podman's data
    /// is.
    pub(super) podman: Podman,
    /// The proxy binary on this host, as podman is handed it for the
    /// bind: the host path on Linux, the machine's view elsewhere.
    /// `None` where the provider runs no containers, and the binary
    /// was not looked for.
    pub(super) proxy_path: Option<String>,
    /// Where stored volumes are loop-mounted, `run/mounts/`, on this
    /// host.
    pub(super) mounts_dir: PathBuf,
    /// Where an ephemeral mount's layer is kept, a file each: the
    /// scratch directory.
    pub(super) scratch_dir: PathBuf,
    /// The Store: where a caller's own images are looked for first,
    /// and where every blob taken from a caller is kept for its proof.
    /// Where the configuration names none, a Store of the deployer's
    /// own with no room, which no identity owns anything in.
    pub(super) store: Arc<Store>,
    /// The provider's registry, which mints a token for every pull from
    /// the Store or from a caller.
    pub(super) registry: Arc<ImageRegistry>,
    /// The port podman reaches the provider's registry at: the
    /// registry's own on Linux, where podman pulls on this host; the
    /// tunnel's elsewhere, inside the machine, where podman pulls.
    pub(super) registry_port: u16,
    /// The label every container of this provider carries,
    /// `diverge.provider=<dir>`, for the cleanup of an earlier life.
    pub(super) label: String,
    /// The volumes, for the mounts.
    pub(super) volumes: Arc<VolumeManager>,
    /// The caps and the image cache, shared with every container.
    pub(super) shared: Arc<Shared>,
    /// The way into the podman machine, on the hosts with one.
    #[cfg(not(target_os = "linux"))]
    pub(super) tunnel: super::Tunnel,
}

impl ContainerDeployer {
    /// A deployer over the `podman` section and the `containers`
    /// section, absent where the provider runs none, with `dir` the
    /// provider's directory, `volumes` its volumes, `registry` its
    /// image registry, `shares` the directories the
    /// volumes live in, which a podman machine must see beside `dir`
    /// and the executable's directory, and `disk` the
    /// `container_overlay_disk` cap, and
    /// `store` the Store the registry serves. Made ready here: on Linux the
    /// provider found to be root, and on the hosts with a machine the
    /// machine brought to what the configuration says, first, since
    /// everything after asks podman; the proxy
    /// binary found beside the executable, where the provider runs
    /// containers; `run/` and `run/mounts/`
    /// made and the auth file written; every container and every
    /// loop mount of an earlier life of this provider swept away;
    /// and, on the hosts with a machine, the tunnel opened. Each
    /// beside the others where nothing orders them.
    pub async fn new(
        podman: Podman,
        containers: Option<Containers>,
        dir: PathBuf,
        volumes: Arc<VolumeManager>,
        registry: Arc<ImageRegistry>,
        shares: Vec<PathBuf>,
        disk: Arc<Limit>,
        store: Arc<Store>,
    ) -> Result<Self, Error> {
        let proxy = proxy_beside_executable()?;
        #[cfg(target_os = "linux")]
        {
            let _ = shares;
            super::root::ensure().await?;
        }
        #[cfg(not(target_os = "linux"))]
        {
            let mut shares = shares;
            shares.push(dir.clone());
            if let Some(beside) = proxy.parent() {
                shares.push(beside.to_path_buf());
            }
            super::machine::ensure(containers.as_ref().map(|containers| containers.memory), &podman.storage_path, &shares).await?;
        }
        let label = format!("diverge.provider={}", dir.display());
        let run_dir = dir.join("run");
        let mounts_dir = run_dir.join("mounts");
        tokio::fs::create_dir_all(&mounts_dir).await?;
        let (proxy_path, ()) = future::try_join(
            async {
                match &containers {
                    Some(_) => proxy_present(&proxy).await.map(Some),
                    None => Ok(None),
                }
            },
            sweep_all(&label, &mounts_dir),
        )
        .await?;
        // After the sweep, which unmounted every layer an earlier life
        // left mounted: the layers' own files, and the directory they
        // are kept in, on a provider that runs containers.
        let scratch_dir = crate::host::volume_manager::scratch_dir(&podman.storage_path);
        if containers.is_some() {
            sweep_layers(&scratch_dir).await?;
        }
        #[cfg(not(target_os = "linux"))]
        let tunnel = super::Tunnel::open(&run_dir, registry.address()).await?;
        #[cfg(not(target_os = "linux"))]
        let registry_port = tunnel.port();
        #[cfg(target_os = "linux")]
        let registry_port = registry.address().port();
        Ok(ContainerDeployer {
            shared: Arc::new(Shared {
                disk,
                memory: Limit::new(containers.as_ref().map_or(0, |containers| containers.memory)),
                images: Images::new(podman.image_cache_disk),
            }),
            scratch_dir,
            podman,
            proxy_path,
            mounts_dir,
            store,
            registry,
            registry_port,
            label,
            volumes,
            #[cfg(not(target_os = "linux"))]
            tunnel,
        })
    }

    /// Everything of this provider's that podman or the host still
    /// holds, gone: every container carrying its label removed at
    /// once, and every loop mount under `run/mounts/` unmounted and
    /// its directory removed. What the construction does, and what a
    /// shutdown does after the connections are gone. A refusal is not
    /// reported, since there is nobody to report it to.
    pub async fn sweep(&self) {
        let _ = sweep_all(&self.label, &self.mounts_dir).await;
    }

    /// The podman machine stopped, on the hosts that have one: what a
    /// shutdown does after its [`sweep`](Self::sweep), so that a
    /// provider that is not running leaves no machine running. On
    /// Linux there is no machine, and this does nothing. A refusal is
    /// not reported, since there is nobody to report it to; the next
    /// start starts the machine or finds it running, either way.
    pub async fn stop_machine(&self) {
        #[cfg(not(target_os = "linux"))]
        let _ = super::machine::stop().await;
    }

    /// Where stored volumes are loop-mounted, `run/mounts/`: what a
    /// build attaches its context volume under, as a run does.
    pub fn mounts_dir(&self) -> &Path {
        &self.mounts_dir
    }

    /// Whether a container runs the image with `id` now.
    pub async fn running_image(&self, id: &str) -> bool {
        self.shared.images.running(id).await
    }

    /// The image with `id` was removed from the cache by something
    /// other than its trim — a delete, a build undone: nothing of it
    /// is recorded any more.
    pub async fn forget_image(&self, id: &str) {
        self.shared.images.forget(id).await;
    }

    /// The port podman reaches the provider's registry at: the
    /// registry's own on Linux, where podman pulls on this host; the
    /// tunnel's elsewhere, inside the machine, where podman pulls.
    /// What a build is tagged under and a pull by token names.
    pub fn registry_port(&self) -> u16 {
        self.registry_port
    }

    /// What remains of the `memory` cap, in BYTES, as of now: what a
    /// run's `memory` could still be, reckoned against what the
    /// running containers reserved and not against the host's free
    /// memory.
    pub fn memory_room(&self) -> u64 {
        self.shared.memory.room()
    }

    /// What remains of the `container_overlay_disk` cap, in BYTES, as
    /// of now: what a run's `disk` and its ephemeral mounts'
    /// `overlay_disk` could still be together.
    pub fn disk_room(&self) -> u64 {
        self.shared.disk.room()
    }
}

/// The deployer a provider that uses no podman has: none. A run is
/// answered `Unsupported` before it reaches a deployer, and so is an
/// `images::read` on a provider with neither a Store nor a registry,
/// which is every provider without podman; either arriving here all
/// the same is [`Error::NoPodman`].
impl container_deployer::ContainerDeployer for Option<Arc<ContainerDeployer>> {
    type Container = Container;
    type Error = Error;

    async fn deploy(
        &self,
        client_identity: &Identity,
        deployment: &Deployment,
        image: &Image,
        caller: &Caller,
    ) -> Result<Deployed<Container>, Refused<Error>> {
        match self {
            Some(deployer) => container_deployer::ContainerDeployer::deploy(&**deployer, client_identity, deployment, image, caller).await,
            None => Err(Refused::Error(Error::NoPodman)),
        }
    }

    async fn read_file(&self, client_identity: &Identity, image: &Image, file: &str) -> Result<ImageInfo, Error> {
        match self {
            Some(deployer) => container_deployer::ContainerDeployer::read_file(&**deployer, client_identity, image, file).await,
            None => Err(Error::NoPodman),
        }
    }
}

impl container_deployer::ContainerDeployer for ContainerDeployer {
    type Container = Container;
    type Error = Error;

    /// Every reference's name checked as a repository path; then the
    /// deploy — the image obtained beside the volumes bound, and the
    /// container run — with the source the image was taken from. A
    /// failed nonce check is [`Refused::ProofFailed`].
    async fn deploy(
        &self,
        client_identity: &Identity,
        deployment: &Deployment,
        image: &Image,
        caller: &Caller,
    ) -> Result<Deployed<Container>, Refused<Error>> {
        if let Some(reference) = image.references.iter().find(|reference| !name_ok(&reference.name)) {
            return Err(Refused::Error(Error::Name(reference.name.clone())));
        }
        match deploy(self, client_identity, deployment, image, caller).await {
            Ok((container, source)) => Ok(Deployed { container, source }),
            Err(Error::ProofFailed) => Err(Refused::ProofFailed),
            Err(error) => Err(Refused::Error(error)),
        }
    }

    /// The image obtained as a deploy obtains it, less the caller, and
    /// the file copied out of a container made of it and never started.
    async fn read_file(&self, client_identity: &Identity, image: &Image, file: &str) -> Result<ImageInfo, Error> {
        read_file(self, client_identity, image, file).await
    }
}

/// Where `diverge-container-proxy` is: beside the provider's
/// executable.
fn proxy_beside_executable() -> Result<PathBuf, Error> {
    let executable = std::env::current_exe()?;
    Ok(executable
        .parent()
        .map(|dir| dir.join("diverge-container-proxy"))
        .unwrap_or_else(|| PathBuf::from("diverge-container-proxy")))
}

/// The proxy, which has to be there, as podman is handed it.
async fn proxy_present(proxy: &Path) -> Result<String, Error> {
    if tokio::fs::metadata(proxy).await.is_err() {
        return Err(Error::Missing(proxy.to_path_buf()));
    }
    Ok(tool_path(proxy))
}

/// The directory the ephemeral mounts' layers are kept in made, and
/// every file an earlier life left in it removed: a layer is a
/// container's, and no container of an earlier life is running. A
/// file that will not go is left, as a mount that will not unmount
/// is.
async fn sweep_layers(scratch_dir: &Path) -> Result<(), Error> {
    tokio::fs::create_dir_all(scratch_dir).await?;
    let mut entries = tokio::fs::read_dir(scratch_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let _ = tokio::fs::remove_file(entry.path()).await;
    }
    Ok(())
}

/// Every container carrying `label` removed at once, then every
/// directory under `mounts_dir` unmounted and removed, each beside
/// every other; a directory the tool would not unmount is removed
/// anyway where the host lets it go.
async fn sweep_all(label: &str, mounts_dir: &Path) -> Result<(), Error> {
    let orphans = podman::containers(label).await.map_err(Error::Podman)?;
    if !orphans.is_empty() {
        let mut args = vec!["rm", "--force", "--time", "0"];
        args.extend(orphans.iter().map(String::as_str));
        podman::podman(args)
            .await
            .map_err(Error::Podman)?
            .require("podman", |status| status.success())
            .map_err(Error::Podman)?;
    }
    let mut entries = tokio::fs::read_dir(mounts_dir).await?;
    let mut dirs = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        dirs.push(entry.path());
    }
    future::join_all(dirs.iter().map(|dir| async move {
        let _ = mount::unmount(&tool_path(dir)).await;
        let _ = tokio::fs::remove_dir(dir).await;
    }))
    .await;
    Ok(())
}
