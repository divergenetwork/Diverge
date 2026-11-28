//! A request, as a listener is sent it.

use rmcp::model::ContentBlock;
use serde::{Deserialize, Serialize};

use crate::daemon::creator::Creator;
use crate::daemon::endpoints::requests::RequestId;

/// One request made of the listener.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct Asked {
    /// What the request is known by: what an answer to it, and its
    /// withdrawal, name.
    pub id: RequestId,
    /// The request: its content, in order, as its creator wrote it.
    /// The daemon reads none of it.
    pub request: Vec<ContentBlock>,
    /// Who made the request: an account, by its identity, or an agent
    /// or a tool, as the daemon names a party — see [`Creator`]. Whom an answer
    /// goes to.
    pub from: Creator,
}
