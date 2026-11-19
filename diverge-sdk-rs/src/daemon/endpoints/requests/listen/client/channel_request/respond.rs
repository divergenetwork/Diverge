//! An answer to a request.

use serde::{Deserialize, Serialize};

use crate::daemon::endpoints::requests::RequestId;

/// The caller's answer to one request it was sent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct Respond {
    /// Which request, by the id it came with.
    pub id: RequestId,
    /// The answer: any JSON value. The daemon reads none of it, and
    /// the creator is sent it as it is.
    pub response: serde_json::Value,
}
