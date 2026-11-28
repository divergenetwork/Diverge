//! Every connected tool, the agent's attached connected tools, the connected tools listed, or
//! only these.

use std::borrow::Cow;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

use super::{Key, Listed};
use crate::shared::permission::Tags;

/// How far a grant over connected tools reaches: every one there is, the
/// connected tools attached to the agent holding the grant, the connected tools
/// listed, or only those the [`Tags`] let through. Flat on the wire:
/// the string `"any"`, the string `"attached"`, the object
/// `{"tools":[…]}`, or the tags object as it is. No word is an
/// object, and neither object decodes as the other, so the four shapes
/// never collide; a string that is neither word does not decode.
///
/// # `attached` is the connected tools attached to the agent
///
/// The word names nothing and reaches the connected tools on the other side
/// of an attachment from the container that holds the grant, judged
/// at every request, whether or not any of them runs. For an
/// agent, those attached to it directly, and those attached to the
/// template it was made from.
/// Held by a tool's container it reaches nothing: a tool is attached
/// to agents, and no connected tool is attached to a tool. See
/// [`agents::Within`](crate::shared::permission::agents::Within) for
/// the same word the other way.
///
/// # `{"tools":[…]}` is exactly the connected tools listed
///
/// Each a [`Key`], as the kind's [`Listed`] states: it reaches those
/// and no other, whatever their tags or their attachments, judged at
/// every request; one that does not exist, or no longer does, is
/// reached by nothing. An empty list reaches nothing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Within {
    /// Every one: the string `"any"`.
    Any,
    /// The connected tools attached to the agent: the string `"attached"`.
    Attached,
    /// Exactly the connected tools listed: the object `{"tools":[…]}`.
    Tools(Vec<Key>),
    /// Only the connected tools the tags let through, as a test.
    Only(Tags),
}

/// The listed form borrowed, to be written without a copy.
#[derive(Serialize)]
struct ListedRef<'a> {
    tools: &'a [Key],
}

impl Serialize for Within {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Within::Any => serializer.serialize_str("any"),
            Within::Attached => serializer.serialize_str("attached"),
            Within::Tools(tools) => ListedRef { tools }.serialize(serializer),
            Within::Only(only) => only.serialize(serializer),
        }
    }
}

/// The two words, so that each is read as a word and not as tags.
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Word {
    Any,
    Attached,
}

/// The three shapes, tried in order: a word, then the list — the
/// kind's own [`Listed`] — then the tags. The list requires its one
/// member and the tags refuse it, so an object is one or the other and
/// never both.
#[derive(Deserialize)]
#[serde(untagged)]
enum Repr {
    Word(Word),
    Listed(Listed),
    Only(Tags),
}

impl<'de> Deserialize<'de> for Within {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match Repr::deserialize(deserializer)? {
            Repr::Word(Word::Any) => Within::Any,
            Repr::Word(Word::Attached) => Within::Attached,
            Repr::Listed(listed) => Within::Tools(listed.tools),
            Repr::Only(only) => Within::Only(only),
        })
    }
}

/// The four shapes as a schema: one of the words, the list, or the
/// tags' own. Written by hand because the derive follows a derived
/// serde and this type's serde is its own.
impl JsonSchema for Within {
    fn schema_name() -> Cow<'static, str> {
        "ConnectedStandaloneToolsWithin".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "diverge_sdk::shared::permission::connected_standalone_tools::Within".into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        let listed = generator.subschema_for::<Listed>();
        let only = generator.subschema_for::<Tags>();
        json_schema!({
            "anyOf": [
                { "type": "string", "const": "any" },
                { "type": "string", "const": "attached" },
                listed,
                only
            ]
        })
    }
}
