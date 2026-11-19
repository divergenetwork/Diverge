//! A listener the request was sent to.

use serde::{Deserialize, Serialize};

use crate::daemon::creator::Creator;

/// One listener was sent the request, just now: one that was
/// listening when the request was created, or one that came to listen
/// after. Sent once for each listener, and never again for it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct Delivered {
    /// Who was sent it: an account, by its identity, or a tool, as the
    /// daemon names a party — see [`Creator`].
    pub to: Creator,
}
