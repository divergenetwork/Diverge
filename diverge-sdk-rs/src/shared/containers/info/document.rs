//! The image's document, parsed and written.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::{Agent, Chat, FalseOr, ParseError, StructuredDecisions, Tool};
use crate::shared::description::Description;

/// `/.diverge/DIVERGE.md`, parsed: an agent's image's document or a
/// tool's, told apart by the member `type`, which is `agent` or
/// `tool`.
///
/// The frontmatter is one JSON object — raw JSON, which is YAML's flow
/// style and so reads as a frontmatter anywhere one is expected. Its
/// member `type` says which of the two it is, and what else it may
/// hold follows from that: both have `name`, `description` and
/// `schema`, and `frontend` where the program serves one; an agent's has `chat` and `structured_decisions` besides, at
/// least one of them an object; a tool's has neither. No frontmatter
/// has a member that is none of its type's. The body is the image's
/// prose — what it runs and the secrets it declares, kept as the author
/// left it — and is the long account of the image, as `description` is
/// the short one.
///
/// The two are different documents and not one with members left out:
/// an image is asked for its document by its digest, by a caller that
/// does not know which of the two it is, and the answer says. The
/// provider carries and does not read any of it; a caller reads the
/// type to know what the image is, the name to call it something,
/// the description to say what it is and the schema to learn what its
/// program takes, without knowing the image.
///
/// On the wire, and as JSON anywhere, a document is one object: the
/// member `type`, then its variant's own members.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Document {
    /// An agent's image's: `"type": "agent"`. See [`Agent`].
    Agent(Agent),
    /// A tool's image's: `"type": "tool"`. See [`Tool`].
    Tool(Tool),
}

/// An agent's frontmatter as written, less its `type`: these members
/// and no other.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AgentFrontmatter {
    name: String,
    description: Description,
    schema: Value,
    #[serde(default, skip_serializing_if = "is_false")]
    frontend: bool,
    #[serde(default, skip_serializing_if = "FalseOr::is_false")]
    chat: FalseOr<Chat>,
    #[serde(default, skip_serializing_if = "FalseOr::is_false")]
    structured_decisions: FalseOr<StructuredDecisions>,
}

/// A tool's frontmatter as written, less its `type`: these members and
/// no other.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ToolFrontmatter {
    name: String,
    description: Description,
    schema: Value,
    #[serde(default, skip_serializing_if = "is_false")]
    frontend: bool,
}

/// A frontmatter as it is written out: `type` first, then its
/// variant's members.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Frontmatter {
    Agent(AgentFrontmatter),
    Tool(ToolFrontmatter),
}

/// Whether a flag is unset, which a frontmatter leaves out.
fn is_false(flag: &bool) -> bool {
    !*flag
}

/// Whether one line, as [`str::split_inclusive`] on `\n` yields it, is
/// a fence: exactly `---`, with or without the newline that ends it.
fn is_fence(line: &str) -> bool {
    line.strip_suffix('\n').unwrap_or(line) == "---"
}

impl Document {
    /// Read a `/.diverge/DIVERGE.md` from its text.
    ///
    /// The rules, in the order they are checked:
    ///
    /// 1. The text begins with a line that is exactly `---`. A line
    ///    ends at `\n`, so a fence followed by anything else — a
    ///    space, a carriage return — is not one. Otherwise
    ///    [`ParseError::NoFrontmatter`].
    /// 2. The frontmatter is every line after it up to the next line
    ///    that is exactly `---`, which closes it. No such line is
    ///    [`ParseError::Unterminated`].
    /// 3. The frontmatter parses as JSON into one object. Otherwise
    ///    [`ParseError::Json`].
    /// 4. The object has the member `type`, and its value is the
    ///    string `agent` or the string `tool`. A `type` that is absent,
    ///    that is no string, or that is any other string, is
    ///    [`ParseError::Type`].
    /// 5. The object's other members are exactly its type's. Both
    ///    types have `name`, a string, `description`, a string that is
    ///    a [`Description`] — one line, trimmed, of one to
    ///    [`DESCRIPTION_CHARACTERS`](crate::shared::description::DESCRIPTION_CHARACTERS)
    ///    characters — and `schema`, each required; and perhaps
    ///    `frontend`, a boolean, `false` when absent. An
    ///    agent's has, besides, perhaps `chat` and perhaps
    ///    `structured_decisions`, each `false` or an object as
    ///    [`FalseOr`] reads one, `false` when absent. A tool's has
    ///    neither: `chat` or `structured_decisions` in a tool's
    ///    frontmatter, whatever its value, is a member that is none of
    ///    its type's. A member missing, a member that is none of its
    ///    type's, a member of the wrong type, or a `description` that
    ///    is no [`Description`], is [`ParseError::Json`].
    /// 6. `schema` is an object, since a JSON Schema is one; any other
    ///    value is [`ParseError::NotObject`].
    /// 7. In an agent's, at least one of `chat` and
    ///    `structured_decisions` is an object. Both `false`, both
    ///    absent, or one `false` and the other absent, is
    ///    [`ParseError::Neither`].
    /// 8. The body is everything after the closing line, as the author
    ///    left it: nothing is trimmed, and a document whose closing
    ///    line is its last has an empty body.
    ///
    /// The text's size is not among them: whoever reads the file out
    /// of an image holds it to [`MAX_BYTES`](super::MAX_BYTES) before
    /// this is called.
    pub fn parse(text: &str) -> Result<Document, ParseError> {
        let mut lines = text.split_inclusive('\n');
        let opening = lines.next().ok_or(ParseError::NoFrontmatter)?;
        if !is_fence(opening) {
            return Err(ParseError::NoFrontmatter);
        }
        let start = opening.len();
        let mut end = start;
        loop {
            let line = lines.next().ok_or(ParseError::Unterminated)?;
            if is_fence(line) {
                let body = text[end + line.len()..].to_owned();
                return Document::of(&text[start..end], body);
            }
            end += line.len();
        }
    }

    /// Rules 3 to 7 of [`parse`](Self::parse), over the frontmatter's
    /// text, with the body it closes over.
    fn of(frontmatter: &str, body: String) -> Result<Document, ParseError> {
        // Rule 3.
        let mut members: Map<String, Value> = serde_json::from_str(frontmatter).map_err(ParseError::Json)?;
        // Rule 4.
        let kind = match members.remove("type") {
            Some(Value::String(kind)) => kind,
            _ => return Err(ParseError::Type),
        };
        // Rules 5 to 7.
        Ok(match kind.as_str() {
            "agent" => {
                let AgentFrontmatter { name, description, schema, frontend, chat, structured_decisions } =
                    serde_json::from_value(Value::Object(members)).map_err(ParseError::Json)?;
                if !schema.is_object() {
                    return Err(ParseError::NotObject);
                }
                if chat.is_false() && structured_decisions.is_false() {
                    return Err(ParseError::Neither);
                }
                Document::Agent(Agent { name, description, schema, frontend, chat, structured_decisions, body })
            }
            "tool" => {
                let ToolFrontmatter { name, description, schema, frontend } = serde_json::from_value(Value::Object(members)).map_err(ParseError::Json)?;
                if !schema.is_object() {
                    return Err(ParseError::NotObject);
                }
                Document::Tool(Tool { name, description, schema, frontend, body })
            }
            _ => return Err(ParseError::Type),
        })
    }

    /// Write the document back as its text: `---`, the frontmatter —
    /// `type` first, then the name, the description, the schema, `frontend` when it is
    /// `true`, and, in an agent's, `chat`
    /// and `structured_decisions` when they are not `false` —
    /// pretty-printed, `---`, and the body, each fence on a line of its
    /// own. One shape is written and read: the assembler that makes an
    /// image's file and any later writer use this, and
    /// [`parse`](Self::parse) of what it writes is this document again,
    /// member for member, for any document [`parse`](Self::parse)
    /// yields: a pretty-printed object has no line that is exactly
    /// `---`, and the body is copied verbatim.
    ///
    /// The `Result` is serde's: serializing a string and a [`Value`]
    /// does not fail, and the signature carries what the serializer
    /// says rather than asserting it.
    pub fn render(&self) -> Result<String, serde_json::Error> {
        let frontmatter = match self {
            Document::Agent(agent) => Frontmatter::Agent(AgentFrontmatter {
                name: agent.name.clone(),
                description: agent.description.clone(),
                schema: agent.schema.clone(),
                frontend: agent.frontend,
                chat: agent.chat,
                structured_decisions: agent.structured_decisions,
            }),
            Document::Tool(tool) => Frontmatter::Tool(ToolFrontmatter {
                name: tool.name.clone(),
                description: tool.description.clone(),
                schema: tool.schema.clone(),
                frontend: tool.frontend,
            }),
        };
        let frontmatter = serde_json::to_string_pretty(&frontmatter)?;
        Ok(format!("---\n{frontmatter}\n---\n{}", self.body()))
    }

    /// The image's own name, whichever type it is.
    pub fn name(&self) -> &str {
        match self {
            Document::Agent(agent) => &agent.name,
            Document::Tool(tool) => &tool.name,
        }
    }

    /// What the image is, in one line, whichever type it is.
    pub fn description(&self) -> &Description {
        match self {
            Document::Agent(agent) => &agent.description,
            Document::Tool(tool) => &tool.description,
        }
    }

    /// The JSON Schema of the image's arguments, whichever type it is.
    pub fn schema(&self) -> &Value {
        match self {
            Document::Agent(agent) => &agent.schema,
            Document::Tool(tool) => &tool.schema,
        }
    }

    /// Whether the program serves a frontend, whichever type it is.
    pub fn frontend(&self) -> bool {
        match self {
            Document::Agent(agent) => agent.frontend,
            Document::Tool(tool) => tool.frontend,
        }
    }

    /// The prose beneath the frontmatter, whichever type it is.
    pub fn body(&self) -> &str {
        match self {
            Document::Agent(agent) => &agent.body,
            Document::Tool(tool) => &tool.body,
        }
    }

    /// The agent's document, where this is one; `None` for a tool's.
    pub fn agent(&self) -> Option<&Agent> {
        match self {
            Document::Agent(agent) => Some(agent),
            Document::Tool(_) => None,
        }
    }
}
