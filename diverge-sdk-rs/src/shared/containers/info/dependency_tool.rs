//! A dependency tool's image's document.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Mode, Scope, Secret};
use crate::shared::description::Description;

/// The document of an image whose program is a dependency tool: its
/// `type` is `dependency_tool`. A dependency tool is one an agent's
/// program declares for itself at its registration, see
/// [`dependencies`](crate::shared::containers::dependencies), and is
/// made by nothing else: a tool template of the image is refused. What
/// a [`StandaloneTool`](super::StandaloneTool)'s has, and the
/// [`Scope`] it is shared by. It has no `chat` and no
/// `structured_decisions`, which are an agent's: a frontmatter that
/// states either, whatever its value, is not this document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DependencyTool {
    /// The image's own name, as its author states it: what a
    /// repository may be named, which [`named`](super::named) states
    /// and [`Document::parse`](super::Document::parse) holds it to.
    /// Nothing makes it unique: two images may say the same name, and
    /// the digest is still what an image is.
    pub name: String,
    /// What the image is, in one line, as its author states it: a
    /// [`Description`], and so at most
    /// [`DESCRIPTION_CHARACTERS`](crate::shared::description::DESCRIPTION_CHARACTERS)
    /// characters of plain text. The one description an image has: it
    /// is the image's own, read out of it wherever the image is, and
    /// no party keeps another beside the image.
    pub description: Description,
    /// The JSON Schema of the image's
    /// [`arguments`](crate::shared::containers::request::Container::arguments),
    /// as the frontmatter states it: a JSON object, since a schema is
    /// one and [`Document::parse`](super::Document::parse) refuses
    /// anything else. A [`Value`] rather than a typed schema, because
    /// a schema is a document in its own vocabulary and this crate has
    /// no business restating it.
    pub schema: Value,
    /// The memory the image's program needs, in BYTES: see
    /// [`Agent::memory`](super::Agent::memory), which this is, member
    /// for member.
    pub memory: u64,
    /// The bytes the image's program needs to write, in BYTES: see
    /// [`Agent::disk`](super::Agent::disk), which this is, member for
    /// member.
    pub disk: u64,
    /// Every secret the image's program could take, each by its name
    /// and what it is: see [`Agent::secrets`](super::Agent::secrets),
    /// which this is, member for member. Which of them a tool needs is
    /// its declaration's to say, by name; the value of each is
    /// resolved through the vault key the agent the tool is spawned
    /// for binds the same name to.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secrets: Vec<Secret>,
    /// How every tool made from the image is kept running: lazy or
    /// eager. See [`Mode`]. Required.
    pub mode: Mode,
    /// How far every tool made from the image is shared, and its
    /// database with it: one per agent, one per agent template, or one
    /// for all. See [`Scope`]. Required.
    pub scope: Scope,
    /// Whether the program serves a frontend beneath `/frontend` (see
    /// [`frontend`](crate::shared::containers::frontend)): `true` only
    /// where the frontmatter says so. Absent, `false`.
    #[serde(default, skip_serializing_if = "is_false")]
    pub frontend: bool,
    /// Everything after the closing `---` line, verbatim: leading
    /// blank lines, trailing newline and all.
    pub body: String,
}

/// Whether a flag is unset, which a frontmatter leaves out.
fn is_false(flag: &bool) -> bool {
    !*flag
}
