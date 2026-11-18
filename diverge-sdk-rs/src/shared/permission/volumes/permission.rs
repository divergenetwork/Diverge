//! One grant's worth over volumes.

use serde::{Deserialize, Serialize};

use crate::shared::permission::{Reach, Tagging, Within};
use super::{Listed, Make, Over};

/// What one grant over volumes allows, in one of three shapes told
/// apart on the wire by their very form, as
/// [`permission`](crate::shared::permission) states them: a bare array
/// of [`Make`] actions; an object of [`Over`] actions and how far they
/// reach; or an object of tagging actions, how far they
/// reach, and which tags. An object that is none of these — an action
/// of one shape among another's, a member missing, a `within` that is
/// not `"any"` or a [`Tags`](crate::shared::permission::Tags) — does not decode.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
pub enum Permission {
    /// To make: the [`Make`] actions held, as a bare array. Nothing is
    /// judged but that the action is held.
    Make(Vec<Make>),
    /// Over volumes that exist: the [`Over`] actions held, reaching
    /// every one of them, or only those the [`Tags`](crate::shared::permission::Tags) let through,
    /// read as a test.
    Over {
        /// The actions.
        actions: Vec<Over>,
        /// How far: `"any"`, the volumes listed, or the tags as a test.
        within: Reach<Listed>,
    },
    /// Tagging volumes: putting tags on, taking them off, or both,
    /// reaching every one or only those the [`Tags`](crate::shared::permission::Tags) let through, and
    /// touching any tag or only the ones listed.
    Tags {
        /// `tag`, `untag`, or both.
        actions: Vec<Tagging>,
        /// How far: `"any"`, the volumes listed, or the tags as a test.
        within: Reach<Listed>,
        /// Which tags: `"any"`, or only these.
        tags: Within<Vec<String>>,
    },
}
