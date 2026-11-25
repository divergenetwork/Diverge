//! The image's document, parsed and written.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::{Agent, Chat, DependencyTool, FalseOr, Mode, ParseError, Scope, StandaloneTool, StructuredDecisions};
use crate::shared::description::Description;

/// `/.diverge/DIVERGE.md`, parsed: an agent's image's document, a
/// standalone tool's or a dependency tool's, told apart by the member
/// `type`, which is `agent`, `standalone_tool` or `dependency_tool`.
///
/// The frontmatter is one JSON object — raw JSON, which is YAML's flow
/// style and so reads as a frontmatter anywhere one is expected. Its
/// member `type` says which of the three it is, and what else it may
/// hold follows from that. All three have `name`, `description`,
/// `schema`, `memory` and `disk`; `secrets` where the program needs
/// any; and `frontend` where the program serves one. An agent's has
/// `chat` and `structured_decisions` besides, at least one of them an
/// object. A standalone tool's has `mode` besides. A dependency tool's
/// has `mode` and `scope` besides. No frontmatter has a member that is
/// none of its type's. The body is the image's prose — what it runs,
/// kept as the author left it — and is the long account of the image,
/// as `description` is the short one.
///
/// An image is one of the three and never two. What kind of tool a
/// tool's image is, is the image's to say and nobody else's: a
/// standalone tool is made by hand from a tool template and is no
/// agent's dependency, and a dependency tool is declared by an agent's
/// program and is made by nothing else.
///
/// The three are different documents and not one with members left
/// out: an image is asked for its document by its digest, by a caller
/// that does not know which of the three it is, and the answer says.
/// The provider carries and does not read any of it; a caller reads
/// the type to know what the image is, the name to call it something,
/// the description to say what it is, the schema to learn what its
/// program takes, and the rest to learn what a container of it needs
/// and how it is kept, without knowing the image.
///
/// On the wire, and as JSON anywhere, a document is one object: the
/// member `type`, then its variant's own members.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Document {
    /// An agent's image's: `"type": "agent"`. See [`Agent`].
    Agent(Agent),
    /// A standalone tool's image's: `"type": "standalone_tool"`. See
    /// [`StandaloneTool`].
    StandaloneTool(StandaloneTool),
    /// A dependency tool's image's: `"type": "dependency_tool"`. See
    /// [`DependencyTool`].
    DependencyTool(DependencyTool),
}

/// An agent's frontmatter as written, less its `type`: these members
/// and no other.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AgentFrontmatter {
    name: String,
    description: Description,
    schema: Value,
    memory: u64,
    disk: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    secrets: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    frontend: bool,
    #[serde(default, skip_serializing_if = "FalseOr::is_false")]
    chat: FalseOr<Chat>,
    #[serde(default, skip_serializing_if = "FalseOr::is_false")]
    structured_decisions: FalseOr<StructuredDecisions>,
}

/// A standalone tool's frontmatter as written, less its `type`: these
/// members and no other.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StandaloneToolFrontmatter {
    name: String,
    description: Description,
    schema: Value,
    memory: u64,
    disk: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    secrets: Vec<String>,
    mode: Mode,
    #[serde(default, skip_serializing_if = "is_false")]
    frontend: bool,
}

/// A dependency tool's frontmatter as written, less its `type`: these
/// members and no other.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DependencyToolFrontmatter {
    name: String,
    description: Description,
    schema: Value,
    memory: u64,
    disk: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    secrets: Vec<String>,
    mode: Mode,
    scope: Scope,
    #[serde(default, skip_serializing_if = "is_false")]
    frontend: bool,
}

/// A frontmatter as it is written out: `type` first, then its
/// variant's members.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Frontmatter {
    Agent(AgentFrontmatter),
    StandaloneTool(StandaloneToolFrontmatter),
    DependencyTool(DependencyToolFrontmatter),
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

/// Rules 6 and 7 of [`Document::parse`]: `schema` is an object, and
/// `secrets` names each secret once, by a name that is not empty.
fn held(schema: &Value, secrets: &[String]) -> Result<(), ParseError> {
    if !schema.is_object() {
        return Err(ParseError::NotObject);
    }
    let alike = |index: usize, name: &String| secrets[..index].contains(name);
    if secrets.iter().enumerate().any(|(index, name)| name.is_empty() || alike(index, name)) {
        return Err(ParseError::Secrets);
    }
    Ok(())
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
    ///    string `agent`, the string `standalone_tool` or the string
    ///    `dependency_tool`. A `type` that is absent, that is no
    ///    string, or that is any other string — `tool` among them — is
    ///    [`ParseError::Type`].
    /// 5. The object's other members are exactly its type's.
    ///    - All three types have `name`, a string; `description`, a
    ///      string that is a [`Description`] — one line, trimmed, of
    ///      one to
    ///      [`DESCRIPTION_CHARACTERS`](crate::shared::description::DESCRIPTION_CHARACTERS)
    ///      characters; `schema`; `memory` and `disk`, each a whole
    ///      number of bytes from 0 to 2 to the power 64 less 1; all
    ///      five required. And perhaps `secrets`, an array of strings,
    ///      empty when absent; and perhaps `frontend`, a boolean,
    ///      `false` when absent.
    ///    - An agent's has, besides, perhaps `chat` and perhaps
    ///      `structured_decisions`, each `false` or an object as
    ///      [`FalseOr`] reads one, `false` when absent.
    ///    - A standalone tool's has, besides, `mode`, the string `lazy`
    ///      or the string `eager`, required.
    ///    - A dependency tool's has, besides, `mode`, the same, and
    ///      `scope`, the string `instance`, `template` or `global`,
    ///      both required.
    ///
    ///    A member of another type's is a member that is none of this
    ///    type's, whatever its value: `chat` or `structured_decisions`
    ///    in a tool's, `mode` or `scope` in an agent's, `scope` in a
    ///    standalone tool's. A member missing, a member that is none
    ///    of its type's, a member of the wrong type, or a
    ///    `description` that is no [`Description`], is
    ///    [`ParseError::Json`].
    /// 6. `schema` is an object, since a JSON Schema is one; any other
    ///    value is [`ParseError::NotObject`].
    /// 7. No name in `secrets` is empty, and no two are alike.
    ///    Otherwise [`ParseError::Secrets`].
    /// 8. In an agent's, at least one of `chat` and
    ///    `structured_decisions` is an object. Both `false`, both
    ///    absent, or one `false` and the other absent, is
    ///    [`ParseError::Neither`].
    /// 9. The body is everything after the closing line, as the author
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

    /// Rules 3 to 8 of [`parse`](Self::parse), over the frontmatter's
    /// text, with the body it closes over.
    fn of(frontmatter: &str, body: String) -> Result<Document, ParseError> {
        // Rule 3.
        let mut members: Map<String, Value> = serde_json::from_str(frontmatter).map_err(ParseError::Json)?;
        // Rule 4.
        let kind = match members.remove("type") {
            Some(Value::String(kind)) => kind,
            _ => return Err(ParseError::Type),
        };
        let members = Value::Object(members);
        // Rules 5 to 8.
        Ok(match kind.as_str() {
            "agent" => {
                let AgentFrontmatter { name, description, schema, memory, disk, secrets, frontend, chat, structured_decisions } =
                    serde_json::from_value(members).map_err(ParseError::Json)?;
                held(&schema, &secrets)?;
                if chat.is_false() && structured_decisions.is_false() {
                    return Err(ParseError::Neither);
                }
                Document::Agent(Agent { name, description, schema, memory, disk, secrets, frontend, chat, structured_decisions, body })
            }
            "standalone_tool" => {
                let StandaloneToolFrontmatter { name, description, schema, memory, disk, secrets, mode, frontend } =
                    serde_json::from_value(members).map_err(ParseError::Json)?;
                held(&schema, &secrets)?;
                Document::StandaloneTool(StandaloneTool { name, description, schema, memory, disk, secrets, mode, frontend, body })
            }
            "dependency_tool" => {
                let DependencyToolFrontmatter { name, description, schema, memory, disk, secrets, mode, scope, frontend } =
                    serde_json::from_value(members).map_err(ParseError::Json)?;
                held(&schema, &secrets)?;
                Document::DependencyTool(DependencyTool { name, description, schema, memory, disk, secrets, mode, scope, frontend, body })
            }
            _ => return Err(ParseError::Type),
        })
    }

    /// Write the document back as its text: `---`, the frontmatter —
    /// `type` first, then the name, the description, the schema, the
    /// memory, the disk, `secrets` when there are any, in a tool's
    /// `mode`, in a dependency tool's `scope`, `frontend` when it is
    /// `true`, and, in an agent's, `chat` and `structured_decisions`
    /// when they are not `false` — pretty-printed, `---`, and the
    /// body, each fence on a line of its own. One shape is written and
    /// read: the assembler that makes an image's file and any later
    /// writer use this, and [`parse`](Self::parse) of what it writes
    /// is this document again, member for member, for any document
    /// [`parse`](Self::parse) yields: a pretty-printed object has no
    /// line that is exactly `---`, and the body is copied verbatim.
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
                memory: agent.memory,
                disk: agent.disk,
                secrets: agent.secrets.clone(),
                frontend: agent.frontend,
                chat: agent.chat,
                structured_decisions: agent.structured_decisions,
            }),
            Document::StandaloneTool(tool) => Frontmatter::StandaloneTool(StandaloneToolFrontmatter {
                name: tool.name.clone(),
                description: tool.description.clone(),
                schema: tool.schema.clone(),
                memory: tool.memory,
                disk: tool.disk,
                secrets: tool.secrets.clone(),
                mode: tool.mode,
                frontend: tool.frontend,
            }),
            Document::DependencyTool(tool) => Frontmatter::DependencyTool(DependencyToolFrontmatter {
                name: tool.name.clone(),
                description: tool.description.clone(),
                schema: tool.schema.clone(),
                memory: tool.memory,
                disk: tool.disk,
                secrets: tool.secrets.clone(),
                mode: tool.mode,
                scope: tool.scope,
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
            Document::StandaloneTool(tool) => &tool.name,
            Document::DependencyTool(tool) => &tool.name,
        }
    }

    /// What the image is, in one line, whichever type it is.
    pub fn description(&self) -> &Description {
        match self {
            Document::Agent(agent) => &agent.description,
            Document::StandaloneTool(tool) => &tool.description,
            Document::DependencyTool(tool) => &tool.description,
        }
    }

    /// The JSON Schema of the image's arguments, whichever type it is.
    pub fn schema(&self) -> &Value {
        match self {
            Document::Agent(agent) => &agent.schema,
            Document::StandaloneTool(tool) => &tool.schema,
            Document::DependencyTool(tool) => &tool.schema,
        }
    }

    /// The memory the image's program needs, in bytes, whichever type
    /// it is.
    pub fn memory(&self) -> u64 {
        match self {
            Document::Agent(agent) => agent.memory,
            Document::StandaloneTool(tool) => tool.memory,
            Document::DependencyTool(tool) => tool.memory,
        }
    }

    /// The bytes the image's program needs to write, whichever type it
    /// is.
    pub fn disk(&self) -> u64 {
        match self {
            Document::Agent(agent) => agent.disk,
            Document::StandaloneTool(tool) => tool.disk,
            Document::DependencyTool(tool) => tool.disk,
        }
    }

    /// The secrets the image needs, by name, in the order it states
    /// them, whichever type it is. Empty where it needs none.
    pub fn secrets(&self) -> &[String] {
        match self {
            Document::Agent(agent) => &agent.secrets,
            Document::StandaloneTool(tool) => &tool.secrets,
            Document::DependencyTool(tool) => &tool.secrets,
        }
    }

    /// How a tool made from the image is kept running, where the image
    /// is a tool's, of either kind; `None` for an agent's.
    pub fn mode(&self) -> Option<Mode> {
        match self {
            Document::Agent(_) => None,
            Document::StandaloneTool(tool) => Some(tool.mode),
            Document::DependencyTool(tool) => Some(tool.mode),
        }
    }

    /// How far a tool made from the image is shared, where the image
    /// is a dependency tool's; `None` for a standalone tool's and for
    /// an agent's.
    pub fn scope(&self) -> Option<Scope> {
        match self {
            Document::Agent(_) | Document::StandaloneTool(_) => None,
            Document::DependencyTool(tool) => Some(tool.scope),
        }
    }

    /// Whether the program serves a frontend, whichever type it is.
    pub fn frontend(&self) -> bool {
        match self {
            Document::Agent(agent) => agent.frontend,
            Document::StandaloneTool(tool) => tool.frontend,
            Document::DependencyTool(tool) => tool.frontend,
        }
    }

    /// The prose beneath the frontmatter, whichever type it is.
    pub fn body(&self) -> &str {
        match self {
            Document::Agent(agent) => &agent.body,
            Document::StandaloneTool(tool) => &tool.body,
            Document::DependencyTool(tool) => &tool.body,
        }
    }

    /// The agent's document, where this is one; `None` for a tool's.
    pub fn agent(&self) -> Option<&Agent> {
        match self {
            Document::Agent(agent) => Some(agent),
            Document::StandaloneTool(_) | Document::DependencyTool(_) => None,
        }
    }

    /// The standalone tool's document, where this is one.
    pub fn standalone_tool(&self) -> Option<&StandaloneTool> {
        match self {
            Document::StandaloneTool(tool) => Some(tool),
            Document::Agent(_) | Document::DependencyTool(_) => None,
        }
    }

    /// The dependency tool's document, where this is one.
    pub fn dependency_tool(&self) -> Option<&DependencyTool> {
        match self {
            Document::DependencyTool(tool) => Some(tool),
            Document::Agent(_) | Document::StandaloneTool(_) => None,
        }
    }
}
