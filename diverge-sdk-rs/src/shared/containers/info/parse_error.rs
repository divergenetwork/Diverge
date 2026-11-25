//! Why a document's text would not read.

use std::fmt;

/// A `/.diverge/DIVERGE.md` that is not one. Each names the rule of
/// [`Document::parse`](super::Document::parse) the text broke, in the
/// order the rules are checked.
#[derive(Debug)]
pub enum ParseError {
    /// The text does not begin with a line that is exactly `---`.
    NoFrontmatter,
    /// No later line is exactly `---`, so the frontmatter never
    /// closes.
    Unterminated,
    /// The frontmatter is not JSON, is not an object, or is not an
    /// object of exactly its type's members: one is missing, one is
    /// none of its type's, or one is of the wrong type. The message is
    /// the parser's, which quotes no value.
    Json(serde_json::Error),
    /// The frontmatter has no `type`, or its `type` is not the string
    /// `agent`, the string `standalone_tool` or the string
    /// `dependency_tool`.
    Type,
    /// An agent's frontmatter in which neither `chat` nor
    /// `structured_decisions` is an object.
    Neither,
    /// The frontmatter's `schema` is not an object, which a schema is.
    NotObject,
    /// The frontmatter's `secrets` names a secret by what is no
    /// secret's name — empty, too long, of more than one line, or not
    /// trimmed — or names one twice.
    Secrets,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::NoFrontmatter => f.write_str("document does not open with a `---` line"),
            ParseError::Unterminated => f.write_str("document frontmatter is never closed by a `---` line"),
            ParseError::Json(error) => write!(f, "document frontmatter is not what its type's is: {error}"),
            ParseError::Type => f.write_str("document frontmatter does not say its type, `agent`, `standalone_tool` or `dependency_tool`"),
            ParseError::Neither => f.write_str("an agent's document says neither how it chats nor how it makes structured decisions"),
            ParseError::NotObject => f.write_str("document schema is not a JSON object"),
            ParseError::Secrets => f.write_str("document secrets name one by what is no secret's name, or one twice"),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::Json(error) => Some(error),
            ParseError::NoFrontmatter | ParseError::Unterminated | ParseError::Type | ParseError::Neither | ParseError::NotObject | ParseError::Secrets => None,
        }
    }
}
