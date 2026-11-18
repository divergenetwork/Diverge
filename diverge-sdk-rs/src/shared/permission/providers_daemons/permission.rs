//! One grant's worth through providers.

use serde::{Deserialize, Serialize};

use crate::shared::permission::{Reach};
use super::{Listed, Over};

/// What one grant through providers allows, in the one shape
/// [`permission`](crate::shared::permission) states for actions over
/// what exists: an object of [`Over`] actions and how far they reach,
/// by the providers' tags. An object that is not this does not decode.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
pub enum Permission {
    /// Through providers: the [`Over`] actions held, reaching every
    /// provider, or only those whose records the [`Tags`](crate::shared::permission::Tags) let through,
    /// read as a test.
    Over {
        /// The actions.
        actions: Vec<Over>,
        /// How far: `"any"`, the providers listed, or the tags as a test.
        within: Reach<Listed>,
    },
}
