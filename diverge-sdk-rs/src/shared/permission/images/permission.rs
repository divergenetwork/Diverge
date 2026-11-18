//! One grant's worth over images.

use serde::{Deserialize, Serialize};

use super::{Action, Listed};
use crate::shared::permission::{Reach};

/// What one grant over images allows, in the one shape
/// [`permission`](crate::shared::permission) states for actions over
/// what exists: an object of [`Action`]s and how far they reach, by
/// the tags of the providers whose Stores they act on. An object that
/// is not this does not decode, and neither does a bare array of
/// actions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
pub enum Permission {
    /// Over the daemon's Stores: the [`Action`]s held, reaching every
    /// provider's, or only those of the providers whose records the
    /// [`Tags`](crate::shared::permission::Tags) let through, read as a test.
    Over {
        /// The actions.
        actions: Vec<Action>,
        /// How far: `"any"`, the providers listed, or the tags as a test.
        within: Reach<Listed>,
    },
}
