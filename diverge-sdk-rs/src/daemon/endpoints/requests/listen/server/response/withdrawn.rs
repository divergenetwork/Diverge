//! A request wanted no more.

use serde::{Deserialize, Serialize};

use crate::daemon::endpoints::requests::RequestId;

/// One request that is no longer to be answered: another listener
/// answered it, or its creator took it back, or went. An id the
/// listener was never sent is to be ignored.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct Withdrawn {
    /// Which request.
    pub id: RequestId,
}
