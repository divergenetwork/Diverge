//! An agent's image's document.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Chat, FalseOr, Icons, StructuredDecisions};

/// The document of an image whose program is an agent: its `type` is
/// `agent`. What a tool's has, and how the agent's program chats and
/// makes structured decisions.
///
/// An agent does at least one of the two: at least one of `chat` and
/// `structured_decisions` is an object. [`Document::parse`](super::Document::parse)
/// refuses a frontmatter in which both are `false` or absent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Agent {
    /// The image's own name, as its author states it. Nothing makes it
    /// unique: two images may say the same name, and the digest is
    /// still what an image is.
    pub name: String,
    /// The JSON Schema of the image's
    /// [`arguments`](crate::shared::containers::request::Container::arguments),
    /// as the frontmatter states it: a JSON object, since a schema is
    /// one and [`Document::parse`](super::Document::parse) refuses
    /// anything else. A [`Value`] rather than a typed schema, because
    /// a schema is a document in its own vocabulary and this crate has
    /// no business restating it.
    pub schema: Value,
    /// Whether the program serves a frontend beneath `/frontend` (see
    /// [`frontend`](crate::shared::containers::frontend)): `true` only
    /// where the frontmatter says so. Absent, `false`.
    #[serde(default, skip_serializing_if = "is_false")]
    pub frontend: bool,
    /// The icons the image ships, each the name of a file beside the
    /// document: see [`Icons`]. Absent, none.
    #[serde(default, skip_serializing_if = "Icons::is_empty")]
    pub icons: Icons,
    /// Whether the program chats — takes a message and answers it —
    /// and how: `false`, or a [`Chat`]. Never `true`. Absent, `false`.
    #[serde(default, skip_serializing_if = "FalseOr::is_false")]
    pub chat: FalseOr<Chat>,
    /// Whether the program makes structured decisions, and how:
    /// `false`, or a [`StructuredDecisions`]. Never `true`. Absent,
    /// `false`.
    #[serde(default, skip_serializing_if = "FalseOr::is_false")]
    pub structured_decisions: FalseOr<StructuredDecisions>,
    /// Everything after the closing `---` line, verbatim: leading
    /// blank lines, trailing newline and all.
    pub body: String,
}

/// Whether a flag is unset, which a frontmatter leaves out.
fn is_false(flag: &bool) -> bool {
    !*flag
}
