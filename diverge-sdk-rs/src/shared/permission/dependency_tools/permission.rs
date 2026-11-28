//! One grant's worth over dependency templates.

use serde::{Deserialize, Serialize};

use crate::shared::permission::{self, Reach, Tagging};
use super::{Listed, Over, Within};

/// What one grant over dependency templates allows, in one of two
/// shapes told apart on the wire by their very form, as
/// [`permission`](crate::shared::permission) states them: an object
/// of [`Over`] actions and how far they reach; or an object
/// of tagging actions, how far they reach, and which tags. There is
/// no making shape: nothing on the wire makes a dependency template.
/// An object that is neither — an action of one shape among the
/// other's, a member missing, a `within` that is not `"any"`,
/// `"attached"`, `{"tools":[…]}` or a
/// [`Tags`](crate::shared::permission::Tags) — does not decode.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
pub enum Permission {
    /// Over dependency templates that exist: the [`Over`] actions
    /// held, reaching every one of them, the agent's attached ones,
    /// the ones listed, or only those the [`Tags`](crate::shared::permission::Tags)
    /// let through, read as a test.
    Over {
        /// The actions.
        actions: Vec<Over>,
        /// How far: `"any"`, `"attached"`, the dependency tools
        /// listed, or the tags as a test. See [`Within`].
        within: Within,
    },
    /// Tagging dependency templates: putting tags on, taking them off,
    /// or both, reaching every one or only those the [`Tags`](crate::shared::permission::Tags) let
    /// through, and touching any tag or only the ones listed.
    Tags {
        /// `tag`, `untag`, or both.
        actions: Vec<Tagging>,
        /// How far: `"any"`, the dependency tools listed, or the tags as a test.
        within: Reach<Listed>,
        /// Which tags: `"any"`, or only these.
        tags: permission::Within<Vec<String>>,
    },
}
