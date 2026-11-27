//! A template as a get or a list answers it: with its arguments, or
//! without them.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::Template;
use crate::daemon::endpoints::agents::create::client::request::Image;

/// A [`Template`] as an answer carries it: every member of it, the
/// `arguments` only where the request asked for them.
///
/// A template's arguments are what its image is told, and an image
/// may be told a credential: so whether an answer carries them is
/// the request's to say — its `arguments` member, `true` or `false`,
/// required — and the caller's to be allowed, by the `arguments`
/// action over the template. Asked for and allowed, `arguments` is
/// the template's own, verbatim; not asked for, the member is absent,
/// and what is left is not the bytes the id is the hash of. Every
/// other member is the template's, as it was handed in.
///
/// This is an answer's shape alone. A template handed to a create is
/// a [`Template`], whose arguments are required.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct Shown<Type> {
    /// What the family says of itself, flat among the members: its
    /// `type`, `agent` or `tool`.
    #[serde(flatten)]
    pub r#type: Type,
    /// The image: a digest and its references. See [`Image`].
    pub image: Image,
    /// The template's [`extra_memory`](Template::extra_memory), in
    /// bytes.
    pub extra_memory: u64,
    /// The template's [`extra_disk`](Template::extra_disk), in bytes.
    pub extra_disk: u64,
    /// The template's [`arguments`](Template::arguments), where the
    /// request said `"arguments": true`. Absent where it said
    /// `false`: absent is not shown, and never a template without
    /// arguments, which has none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Value>,
}

impl<Type: Clone> Template<Type> {
    /// The template as an answer carries it: whole where `arguments`,
    /// and with the arguments left out where not.
    pub fn shown(&self, arguments: bool) -> Shown<Type> {
        Shown {
            r#type: self.r#type.clone(),
            image: self.image.clone(),
            extra_memory: self.extra_memory,
            extra_disk: self.extra_disk,
            arguments: arguments.then(|| self.arguments.clone()),
        }
    }
}
