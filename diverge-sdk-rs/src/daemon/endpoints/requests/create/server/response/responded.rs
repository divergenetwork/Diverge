//! The request's answer.

use serde::{Deserialize, Serialize};

use crate::daemon::creator::Creator;

/// The one answer the request got.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct Responded {
    /// The answer: any JSON value, as the listener wrote it. The
    /// daemon reads none of it.
    pub response: serde_json::Value,
    /// Who answered: an account, by its identity, or a tool, as the
    /// daemon names a party — see [`Creator`]. One of those the
    /// request was delivered to.
    pub from: Creator,
}
