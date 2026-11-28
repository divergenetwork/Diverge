//! Every one, exactly these, or those the tags let through.

use std::borrow::Cow;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

use super::Tags;

/// How far a grant a container's program declares reaches, over one kind
/// of thing: every one of the kind, exactly the ones listed, or only
/// those the [`Tags`] let through. `L` is the kind's own list — its
/// `Listed`, an object of one member naming the records one by one.
/// The rules:
///
/// 1. **One of three, and never two.** Flat on the wire: the string
///    `"any"`; the kind's list as it is, `{"ids":[…]}`,
///    `{"volumes":[…]}`; or the tags as they are,
///    `{"all_tags":[…],"any_tags":[…]}`. A list has its one member
///    and no other, and the tags refuse it, so an object is one or the
///    other and never both: a grant that listed records and named tags
///    at once would have to say whether it meant both or either, and
///    there is no writing it.
/// 2. **A list reaches exactly what it lists**, whatever the tags of
///    what it lists, judged at every request. A record that does not
///    exist, or no longer does, is reached by nothing. An empty list
///    reaches nothing.
/// 3. **Either is two grants.** A tool that is to reach these records
///    and also everything tagged so declares one grant for each:
///    grants only add, and no grant takes from another.
/// 4. **A string that is not `"any"` does not decode**, and neither
///    does an object that is neither form.
///
/// A list names records, and a record is one daemon's: an image that
/// lists any reads them out of the arguments it was registered with,
/// and declares one list here and another there. See
/// [`permission`](super) for why that keeps an image shareable.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Reach<L> {
    /// Every one: the string `"any"`.
    Any,
    /// Exactly the ones listed, as the kind's list is on the wire.
    Listed(L),
    /// Only those the tags let through, as a test.
    Tags(Tags),
}

impl<L> Default for Reach<L> {
    /// `"any"`: a member left absent reaches everything.
    fn default() -> Self {
        Reach::Any
    }
}

impl<L: Serialize> Serialize for Reach<L> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Reach::Any => serializer.serialize_str("any"),
            Reach::Listed(listed) => listed.serialize(serializer),
            Reach::Tags(tags) => tags.serialize(serializer),
        }
    }
}

/// The one word, so that it is read as a word and not as an object.
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Word {
    Any,
}

/// The three shapes, tried in order: the word, then the list, then
/// the tags. The list requires its one member and the tags refuse it,
/// so an object is one or the other and never both.
#[derive(Deserialize)]
#[serde(untagged)]
enum Repr<L> {
    Word(Word),
    Listed(L),
    Tags(Tags),
}

impl<'de, L: Deserialize<'de>> Deserialize<'de> for Reach<L> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match Repr::deserialize(deserializer)? {
            Repr::Word(Word::Any) => Reach::Any,
            Repr::Listed(listed) => Reach::Listed(listed),
            Repr::Tags(tags) => Reach::Tags(tags),
        })
    }
}

/// The three shapes as a schema: the one word, the list's own, or the
/// tags' own. Written by hand because the derive follows a derived
/// serde and this type's serde is its own.
impl<L: JsonSchema> JsonSchema for Reach<L> {
    fn schema_name() -> Cow<'static, str> {
        format!("Reach_{}", L::schema_name()).into()
    }

    fn schema_id() -> Cow<'static, str> {
        format!("diverge_sdk::shared::permission::Reach<{}>", L::schema_id()).into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        let listed = generator.subschema_for::<L>();
        let tags = generator.subschema_for::<Tags>();
        json_schema!({
            "anyOf": [
                { "type": "string", "const": "any" },
                listed,
                tags
            ]
        })
    }
}
