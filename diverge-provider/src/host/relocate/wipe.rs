//! Podman's data at a storage path, deleted.

use std::path::Path;

use super::{Error, moving};

/// Everything podman keeps at `path` deleted, and everything outside
/// it that names it, so that a provider started on the path finds
/// nothing there and makes all of it again: an empty store on Linux, a
/// machine made new on macOS and Windows. What the CLI's
/// `provider podman storage_path delete` does, with the provider
/// stopped and its lock held. Every image the store or the machine
/// held is gone with it, and is not got back.
///
/// The folder alone deleted would not be enough, for the reasons a
/// folder alone moved is not — see [`relocate`](super). The rules:
///
/// 1. **Linux.** Anything still mounted at or under the path is
///    refused, [`Error::Mounted`], before anything is deleted. Then
///    the folder is removed whole. Podman is not asked: its database
///    is in the folder and goes with it.
/// 2. **macOS.** The machine is stopped if it runs. Then the folder
///    is removed whole; the machine's description and connections are
///    in it.
/// 3. **Windows.** The machine is stopped if it runs, and then, for
///    everything that names the path — see
///    [`forget`](super::machine::forget): podman is asked to remove
///    each machine whose description names it; every WSL distribution
///    whose disk is under it is unregistered, which is WSL deleting
///    the disk; each such description left is deleted; each connection
///    whose key is under it is taken out of the connections. Then the
///    folder is removed whole.
/// 4. **Everything under the path goes**, not the provider's machine
///    alone: a machine another hand made with its data there has its
///    disk there, and is deleted with the rest.
/// 5. **A path that is not there** is nothing to delete, and is not
///    an error; what names it from outside is still removed.
/// 6. **A stop that fails does not stop the delete**: podman missing
///    or broken is a reason to delete, not a reason not to.
/// 7. **It is not undone.** A step that fails is the error, with
///    what came before it deleted; asked again, it deletes the rest.
pub async fn wipe(path: &Path) -> Result<(), Error> {
    #[cfg(target_os = "linux")]
    {
        let mounts = super::linux::mounted(path).await?;
        if !mounts.is_empty() {
            return Err(Error::Mounted(mounts));
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = super::machine::still(path).await;
        super::machine::forget(path).await?;
    }
    moving::remove(path).await
}
