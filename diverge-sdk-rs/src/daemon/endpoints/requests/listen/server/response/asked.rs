//! A request, as a listener is sent it.

use serde::{Deserialize, Serialize};

use crate::daemon::endpoints::requests::RequestId;

/// One request made of the listener.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct Asked {
    /// What the request is known by: what an answer to it, and its
    /// withdrawal, name.
    pub id: RequestId,
    /// The request: any JSON value, as its creator wrote it. The
    /// daemon reads none of it.
    pub request: serde_json::Value,
}
