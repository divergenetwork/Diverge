//! An agent's image's document.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Chat, FalseOr, Secret, StructuredDecisions};
use crate::shared::description::Description;

/// The document of an image whose program is an agent: its `type` is
/// `agent`. What every image says of itself — its name, what it is, what
/// it takes, what it needs — and how the agent's program chats and
/// makes structured decisions. It has no `mode` and no `scope`: those
/// are a tool's.
///
/// An agent does at least one of the two: at least one of `chat` and
/// `structured_decisions` is an object. [`Document::parse`](super::Document::parse)
/// refuses a frontmatter in which both are `false` or absent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Agent {
    /// The image's own name, as its author states it: what a
    /// repository may be named, but that it may have capitals, which
    /// [`named`](super::named) states and
    /// [`Document::parse`](super::Document::parse) holds it to.
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
    /// The memory the image's program needs, in BYTES: what a
    /// container of the image has before anyone gives it more.
    /// Required.
    ///
    /// A container's limit is this and what its template adds — the
    /// `extra_memory` of a
    /// [daemon's template](crate::daemon::template::Template) or of a
    /// [dependency's declaration](crate::shared::containers::dependencies::Template)
    /// — summed, and the sum is the
    /// [`memory`](crate::shared::containers::request::Container::memory)
    /// of the run request. So an image run with nothing added runs
    /// with what it says it needs, and no template says less.
    ///
    /// Bytes rather than megabytes because a unit that has to be
    /// spelled out in prose is a unit half of everyone gets wrong.
    pub memory: u64,
    /// The bytes the image's program needs to WRITE, in BYTES: its
    /// container's own filesystem, over the image it came from, before
    /// anyone gives it more. Required.
    ///
    /// A container's limit is this and its template's `extra_disk`,
    /// summed, as [`memory`](Self::memory) is, and the sum is the
    /// [`disk`](crate::shared::containers::request::Container::disk)
    /// of the run request. The image's layers are read-only and are
    /// not counted, and no mount is.
    pub disk: u64,
    /// Every secret the image's program could take, each by its name
    /// and what it is, in the order its author states them: see
    /// [`Secret`]. All of them, whatever the arguments: which of them
    /// a container needs is its template's to say, by name, since
    /// which a program takes follows from the arguments it is given,
    /// and a template names none the image does not state. Which the
    /// program then takes, and in what mode — read once per
    /// activation, or held and written back — is its own answer at its
    /// registration, see
    /// [`secrets`](crate::shared::containers::secrets). A name is a
    /// secret's name, and no two are alike:
    /// [`Document::parse`](super::Document::parse) refuses a
    /// frontmatter that says otherwise. Absent, the program takes
    /// none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secrets: Vec<Secret>,
    /// Whether the program serves a frontend beneath `/frontend` (see
    /// [`frontend`](crate::shared::containers::frontend)): `true` only
    /// where the frontmatter says so. Absent, `false`.
    #[serde(default, skip_serializing_if = "is_false")]
    pub frontend: bool,
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
