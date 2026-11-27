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
    /// The frontmatter's `name` is no image's name: empty, over
    /// [`NAME_BYTES`](super::NAME_BYTES) bytes, or of anything but
    /// runs of letters and digits joined by separators.
    Name,
    /// The frontmatter's `schema` is not an object, which a schema is.
    NotObject,
    /// An agent's frontmatter in which neither `chat` nor
    /// `structured_decisions` is an object.
    Neither,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::NoFrontmatter => f.write_str("document does not open with a `---` line"),
            ParseError::Unterminated => f.write_str("document frontmatter is never closed by a `---` line"),
            ParseError::Json(error) => write!(f, "document frontmatter is not what its type's is: {error}"),
            ParseError::Type => f.write_str("document frontmatter does not say its type, `agent`, `standalone_tool` or `dependency_tool`"),
            ParseError::Name => f.write_str("document name is no image's name: 1 to 128 bytes, of letters and digits joined by `.`, `_`, `__` or hyphens"),
            ParseError::NotObject => f.write_str("document schema is not a JSON object"),
            ParseError::Neither => f.write_str("an agent's document says neither how it chats nor how it makes structured decisions"),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::Json(error) => Some(error),
            ParseError::NoFrontmatter | ParseError::Unterminated | ParseError::Type | ParseError::Name | ParseError::NotObject | ParseError::Neither => None,
        }
    }
}
