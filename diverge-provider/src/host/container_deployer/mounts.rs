//! The volume mounts of a deploy: checked, resolved, and put where
//! podman can bind them.

use std::path::{Path, PathBuf};

use crate::host::tools::mount;
use crate::host::volume_manager::{format_image, make_directories, reserve_image};
use crate::protocol::mount::Mount;
use diverge_sdk::provider::endpoints::volumes::Mode;
use crate::protocol::volume_manager::VolumeManager as _;
use futures_util::future;

use super::{ContainerDeployer, Error};
use crate::host::volume_manager::Volume;

/// One mount made ready: the `--volume` argument podman is handed,
/// and the volume it binds, attached, to be detached when the
/// container is gone.
#[derive(Debug, Clone)]
pub struct Bound {
    /// `<host>:<container>`, with `:O` and where its layer is after it
    /// for an ephemeral volume — podman's overlay, the container's
    /// writes discarded with it — and `:ro` for a read-only one.
    pub argument: String,
    /// The volume, attached once for this mount.
    pub volume: Volume,
    /// An ephemeral mount's layer, where it has one: see [`Layer`].
    pub layer: Option<Layer>,
}

/// The layer an ephemeral mount's writes land in: an image of exactly
/// the mount's `overlay_disk`, mounted, for the container's life. The
/// rules:
///
/// 1. **Of exactly the bytes reserved.** The image is made
///    `overlay_disk` long and formatted to fill it, so what the
///    container writes into the volume cannot pass what the mount
///    reserved: a write past it fails inside the container as a
///    write to a full disk does.
/// 2. **Where podman is told.** The image is mounted as a stored
///    volume's is, and holds two directories, `upper` and `work`,
///    which the mount names to podman as the overlay's `upperdir` and
///    `workdir`. Left to itself podman keeps an overlay's writes in
///    its own storage, bounded by nothing.
/// 3. **Gone with the container.** [`release`] unmounts it and
///    removes the image, so nothing written is kept, which is what
///    the mode means. One left by an earlier life is unmounted with
///    the loop mounts and removed with the scratch, at the start.
/// 4. **None for nothing.** A mount whose `overlay_disk` is `0` has
///    no layer and is bound read-only: nothing can be written into
///    it, which is what `0` reserves.
#[derive(Debug, Clone)]
pub struct Layer {
    /// The image, on this host.
    image: PathBuf,
    /// Where it is mounted, as the tool sees it.
    dir: String,
}

/// Every mount checked and made ready, each resolved beside every
/// other. A mount that could not be made undoes the ones that were,
/// and the first failure is the error. Nothing is asked of podman.
pub async fn bind(deployer: &ContainerDeployer, mounts: &[Mount]) -> Result<Vec<Bound>, Error> {
    check(mounts)?;
    let outcomes = future::join_all(mounts.iter().map(|mount| one(deployer, mount))).await;
    let mut bound = Vec::with_capacity(outcomes.len());
    let mut failed = None;
    for outcome in outcomes {
        match outcome {
            Ok(one) => bound.push(one),
            Err(error) => failed = failed.or(Some(error)),
        }
    }
    match failed {
        None => Ok(bound),
        Some(error) => {
            release(&bound).await;
            Err(error)
        }
    }
}

/// Every volume of `bound` detached, each beside every other: the
/// last container to have a stored volume unmounts its image; and
/// every ephemeral mount's layer unmounted and removed.
pub async fn release(bound: &[Bound]) {
    future::join_all(bound.iter().map(|one| async move {
        future::join(one.volume.detach(), async {
            if let Some(layer) = &one.layer {
                layer.remove().await;
            }
        })
        .await;
    }))
    .await;
}

impl Layer {
    /// Rules 1 and 2 of [`Layer`]: an image of `bytes` in
    /// `scratch_dir`, formatted, its two directories made, mounted
    /// under `mounts_dir`. A step that fails undoes the ones before
    /// it.
    async fn make(scratch_dir: &Path, mounts_dir: &Path, bytes: u64) -> Result<Layer, Error> {
        let name = uuid::Uuid::new_v4().to_string();
        let image = scratch_dir.join(format!("{name}.layer"));
        let made = async {
            reserve_image(&image, bytes).await?;
            format_image(&image, bytes).await?;
            make_directories(&image, &[UPPER, WORK]).await
        }
        .await;
        if let Err(error) = made {
            let _ = tokio::fs::remove_file(&image).await;
            return Err(Error::Volumes(error));
        }
        let dir = tool_path(&mounts_dir.join(name));
        if let Err(error) = mount::mount(&image, &dir, false).await {
            let _ = tokio::fs::remove_file(&image).await;
            return Err(Error::Mount(error));
        }
        Ok(Layer { image, dir })
    }

    /// Rule 3 of [`Layer`]. A refusal is not reported, there being
    /// nobody to report it to: the next start sweeps what is left.
    async fn remove(&self) {
        let _ = mount::unmount(&self.dir).await;
        let _ = tokio::fs::remove_file(&self.image).await;
    }
}

/// The directory of a [`Layer`] the container's writes land in.
const UPPER: &str = "upper";

/// The directory of a [`Layer`] the kernel's overlay works in.
const WORK: &str = "work";

/// The refusals the contract puts on the provider, for what a
/// deployment carries: the root as a container path; a component that
/// is not a name, in either path; two mounts at one path; a path that
/// is a prefix of another's.
fn check(mounts: &[Mount]) -> Result<(), Error> {
    for mount in mounts {
        if mount.container_path.is_empty() {
            return Err(Error::Path("the container's root".to_string()));
        }
        for component in mount.container_path.iter().chain(&mount.volume_relative_path) {
            if !component_ok(component) {
                return Err(Error::Path(format!("the component `{component}` is not a name")));
            }
        }
    }
    for (index, mount) in mounts.iter().enumerate() {
        for other in &mounts[index + 1..] {
            if mount.container_path == other.container_path {
                return Err(Error::Path(format!("two mounts at `/{}`", mount.container_path.join("/"))));
            }
            if mount.container_path.starts_with(&other.container_path)
                || other.container_path.starts_with(&mount.container_path)
            {
                return Err(Error::Path(format!(
                    "`/{}` lies inside `/{}`",
                    mount.container_path.join("/"),
                    other.container_path.join("/")
                )));
            }
        }
    }
    Ok(())
}

/// Whether a path component is a name: not empty, not `.` or `..`,
/// and without a `/` or a NUL in it.
fn component_ok(component: &str) -> bool {
    !component.is_empty() && component != "." && component != ".." && !component.contains(['/', '\0'])
}

/// One mount resolved: the volume looked up for its identity and
/// attached — its image loop-mounted if this is the first container
/// to have it — the relative path descended on the directory it
/// answers, and the argument built: plain for a persistent volume,
/// under podman's overlay for an ephemeral one, read-only for a
/// read-only one. The rules of an ephemeral one's layer:
///
/// 1. **A number exactly when ephemeral.** A mount that states an
///    `overlay_disk` for a volume that is not ephemeral, or none for
///    one that is, is [`Error::Overlay`], before the volume is
///    attached.
/// 2. **Its layer made**, as [`Layer`] states, and named to podman;
///    with `0` reserved there is none, and the mount is read-only.
/// 3. A layer that cannot be made detaches the volume again.
async fn one(deployer: &ContainerDeployer, mount: &Mount) -> Result<Bound, Error> {
    let volume = deployer
        .volumes
        .get(&mount.client_identity, &mount.volume_name)
        .await?
        .ok_or_else(|| Error::Volume(mount.volume_name.clone()))?;
    let mode = volume.mode().await;
    // Rule 1.
    if (mode == Mode::Ephemeral) != mount.overlay_disk.is_some() {
        return Err(Error::Overlay(mount.volume_name.clone()));
    }
    let dir = volume.attach(&deployer.mounts_dir).await.map_err(Error::Mount)?;
    let host = descend(&dir, &mount.volume_relative_path);
    let mut argument = format!("{host}:/{}", mount.container_path.join("/"));
    let mut layer = None;
    match (mode, mount.overlay_disk) {
        (Mode::Persistent, _) => {}
        (Mode::ReadOnly, _) | (Mode::Ephemeral, None | Some(0)) => argument.push_str(":ro"),
        // Rule 2.
        (Mode::Ephemeral, Some(bytes)) => match Layer::make(&deployer.scratch_dir, &deployer.mounts_dir, bytes).await {
            Ok(made) => {
                argument.push_str(&format!(":O,upperdir={}/{UPPER},workdir={}/{WORK}", made.dir, made.dir));
                layer = Some(made);
            }
            // Rule 3.
            Err(error) => {
                volume.detach().await;
                return Err(error);
            }
        },
    }
    Ok(Bound { argument, volume, layer })
}

/// `base` with `components` after it, joined by `/` as the tool
/// joins paths.
pub fn descend(base: &str, components: &[String]) -> String {
    let mut path = base.to_string();
    for component in components {
        path.push('/');
        path.push_str(component);
    }
    path
}

/// A host path as the tool and podman see it: on Linux, the path
/// itself.
#[cfg(target_os = "linux")]
pub fn tool_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// A host path as the tool and podman see it: the machine's view of
/// it.
#[cfg(not(target_os = "linux"))]
pub fn tool_path(path: &Path) -> String {
    crate::host::tools::podman::path(path)
}
