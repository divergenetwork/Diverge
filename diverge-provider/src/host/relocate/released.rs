//! Nothing holding what a volume store keeps.

use std::path::Path;

use super::Error;

/// The volume images of the provider's stores let go of, before a
/// store is moved or deleted with the provider stopped.
///
/// A stored volume is an image file in a store, and while a container
/// has it the image is loop-mounted: under `mounts`, the provider's
/// `run/mounts/`, on Linux; inside the podman machine on macOS and
/// Windows. A provider that stopped unmounted every one and stopped
/// its machine. One that was killed did neither, and an image moved
/// or deleted under a mount is a filesystem torn. So:
///
/// - **Linux.** Anything still mounted at or under `mounts` is
///   refused, [`Error::Mounted`], naming it: the provider's next
///   start sweeps those, and so does unmounting them by hand.
/// - **macOS and Windows.** The provider's machine, kept under
///   `storage`, is stopped if it runs, which ends every mount in it:
///   see [`still`](super::machine::still). `None`, the provider uses
///   no podman, has no machine, and nothing is asked.
pub async fn released(storage: Option<&Path>, mounts: &Path) -> Result<(), Error> {
    #[cfg(target_os = "linux")]
    {
        let _ = storage;
        let held = super::linux::mounted(mounts).await?;
        if !held.is_empty() {
            return Err(Error::Mounted(held));
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = mounts;
        if let Some(storage) = storage {
            super::machine::still(storage).await?;
        }
    }
    Ok(())
}
