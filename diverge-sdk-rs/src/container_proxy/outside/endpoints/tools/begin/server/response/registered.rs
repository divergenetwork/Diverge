//! What the container's program declared when it registered.

use serde::{Deserialize, Serialize};

use crate::shared::permission::Grant;

/// The program's answer to its registration, as `Begun` carries it:
/// the permissions it holds over the daemon, the container's for its
/// life. A tool container declares no dependencies — a list its
/// program answers with is ignored by the proxy, and none rides here.
/// The provider carries them to the caller in one
/// [`registration`](crate::shared::containers::registration) ask when
/// there is at least one, and reads nothing of them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Registered {
    /// What the program may ask of the daemon, as grants, in the order
    /// it declared them: see [`Grant`]. Absent when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<Grant>,
}
