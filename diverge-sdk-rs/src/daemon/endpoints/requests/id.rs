//! A request's name.

use serde::{Deserialize, Serialize};

/// What a request is known by, from its creation to its end: a
/// string the daemon draws at random when the request is created, and
/// which nothing else is read from. A listener answers a request by
/// it, and is told of a withdrawal by it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(transparent)]
pub struct RequestId(pub String);

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
