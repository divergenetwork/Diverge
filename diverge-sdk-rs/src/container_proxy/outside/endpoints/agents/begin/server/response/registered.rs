//! What the container's program declared when it registered.

use serde::{Deserialize, Serialize};

use crate::shared::containers::dependencies::Template;

/// The program's answer to its registration, as `Begun` carries it:
/// the dependencies it declared, each a tool container the caller
/// deploys. They are the container's for its life. The provider
/// carries them to the caller in one
/// [`registration`](crate::shared::containers::registration) ask when
/// there is at least one, and reads none.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Registered {
    /// The dependencies, in the order the program declared them: see
    /// [`dependencies`](crate::shared::containers::dependencies).
    /// Absent when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<Template>,
}
