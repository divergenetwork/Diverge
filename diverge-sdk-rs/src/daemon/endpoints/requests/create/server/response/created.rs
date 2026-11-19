//! The request, made.

use serde::{Deserialize, Serialize};

use crate::daemon::endpoints::requests::RequestId;

/// The request exists: listeners are being sent it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct Created {
    /// What it is known by, drawn at random by the daemon: what each
    /// listener is sent it under.
    pub id: RequestId,
}
