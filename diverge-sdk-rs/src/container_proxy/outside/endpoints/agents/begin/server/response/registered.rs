//! What the container's program declared when it registered.

use serde::{Deserialize, Serialize};

use crate::shared::containers::dependencies::Template;
use crate::shared::permission::Grant;

/// The program's answer to its registration, as `Begun` carries it:
/// the dependencies it declared, each a tool container the caller
/// deploys, and the permissions it holds over the daemon. Both are
/// the container's for its life. The provider carries them to the
/// caller in one
/// [`registration`](crate::shared::containers::registration) ask when
/// either is non-empty, and reads neither.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Registered {
    /// The dependencies, in the order the program declared them: see
    /// [`dependencies`](crate::shared::containers::dependencies).
    /// Absent when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<Template>,
    /// What the program may ask of the daemon, as grants, in the order
    /// it declared them: see [`Grant`]. Absent when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<Grant>,
}
