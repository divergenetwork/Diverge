//! One grant's worth over the compartments.

use serde::{Deserialize, Serialize};

use crate::shared::permission::{Reach, Tagging, Within};
use super::{Listed, Over};

/// What one grant over the compartments allows, in one of two shapes
/// told apart on the wire by their very form, as
/// [`permission`](crate::shared::permission) states them: an object of
/// [`Over`] actions and how far they reach; or an object of tagging
/// actions, how far they reach, and which tags. An object that is
/// neither does not decode.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "PostgresCompartmentsPermission")]
#[serde(untagged, deny_unknown_fields)]
pub enum Permission {
    /// Over compartments: the [`Over`] actions held, reaching every
    /// one of them, those listed, or only those the
    /// [`Tags`](crate::shared::permission::Tags) let through.
    Over {
        /// The actions.
        actions: Vec<Over>,
        /// How far: `"any"`, the compartments listed, or the tags as a test.
        within: Reach<Listed>,
    },
    /// Tagging compartments: putting tags on, taking them off, or
    /// both, reaching as far as `within` says, and touching any tag or
    /// only the ones listed.
    Tags {
        /// `tag`, `untag`, or both.
        actions: Vec<Tagging>,
        /// How far: `"any"`, the compartments listed, or the tags as a test.
        within: Reach<Listed>,
        /// Which tags: `"any"`, or only these.
        tags: Within<Vec<String>>,
    },
}
