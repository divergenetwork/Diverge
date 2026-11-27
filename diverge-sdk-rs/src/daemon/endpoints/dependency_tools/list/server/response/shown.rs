//! A dependency template as a get or a list answers it: with its
//! arguments, or without them.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::shared::containers::dependencies::{Mount, Template};
use crate::shared::containers::request::Image;

/// A dependency [`Template`] as an answer carries it: every member of
/// it, the `arguments` only where the request asked for them.
///
/// A template's arguments are what its image is told, and an image
/// may be told a credential: so whether an answer carries them is
/// the request's to say — its `arguments` member, `true` or `false`,
/// required — and the caller's to be allowed, by the `arguments`
/// action over the tool. Asked for and allowed, `arguments` is the
/// template's own, verbatim; not asked for, the member is absent, and
/// what is left is not the bytes the id is the hash of. Every other
/// member is the template's, as the program declared it.
///
/// This is an answer's shape alone. What a program declares is a
/// [`Template`], whose arguments are required.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "DependencyToolsShown")]
pub struct Shown {
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
    /// The template's
    /// [`fuse_file_mounts`](Template::fuse_file_mounts). Absent when
    /// empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fuse_file_mounts: Vec<Mount>,
    /// The template's
    /// [`fuse_directory_mounts`](Template::fuse_directory_mounts).
    /// Absent when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fuse_directory_mounts: Vec<Mount>,
}

impl Shown {
    /// `template` as an answer carries it: whole where `arguments`,
    /// and with the arguments left out where not.
    pub fn of(template: &Template, arguments: bool) -> Shown {
        Shown {
            image: template.image.clone(),
            extra_memory: template.extra_memory,
            extra_disk: template.extra_disk,
            arguments: arguments.then(|| template.arguments.clone()),
            fuse_file_mounts: template.fuse_file_mounts.clone(),
            fuse_directory_mounts: template.fuse_directory_mounts.clone(),
        }
    }
}
