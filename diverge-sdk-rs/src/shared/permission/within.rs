//! Every one, or only these.

use std::borrow::Cow;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

/// How far a grant reaches, or which tags it may touch: every one there
/// is, or only what `T` names. Flat on the wire: the string `"any"`, or
/// `T` as it is — a filter object, a list of tags. `"any"` is no filter
/// and no list, so the two shapes never collide; a string that is not
/// `"any"` does not decode.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Within<T> {
    /// Every one: the string `"any"`.
    Any,
    /// Only what `T` names, as `T` is on the wire.
    Only(T),
}

impl<T> Within<T> {
    /// Whether this is `"any"`: what a member left absent means.
    pub fn is_any(&self) -> bool {
        matches!(self, Within::Any)
    }
}

impl<T> Default for Within<T> {
    /// `"any"`: a member left absent reaches everything.
    fn default() -> Self {
        Within::Any
    }
}

impl<T: Serialize> Serialize for Within<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Within::Any => serializer.serialize_str("any"),
            Within::Only(only) => only.serialize(serializer),
        }
    }
}

/// The one word, so that it is read as a word and not as a `T`.
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Word {
    Any,
}

/// The two shapes, tried in order: the word, then `T`.
#[derive(Deserialize)]
#[serde(untagged)]
enum Repr<T> {
    Word(Word),
    Only(T),
}

impl<'de, T: Deserialize<'de> + Only> Deserialize<'de> for Within<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match Repr::<T>::deserialize(deserializer)? {
            Repr::Word(Word::Any) => Within::Any,
            Repr::Only(only) => match only.mixed() {
                None => Within::Only(only),
                Some(why) => return Err(serde::de::Error::custom(why)),
            },
        })
    }
}

/// What a [`Within`] may hold as its `T`, and what of it a `within`
/// refuses. The rules, which are the daemon's for every grant's
/// reach:
///
/// 1. **A grant reaches all, exactly these, or by tags, and never two
///    of them.** `"any"` is all. A `T` that names records reaches
///    exactly those; a `T` that names tags reaches by them.
/// 2. **Naming records and naming tags together is refused**, and so
///    is naming records by two different members at once: such a
///    `within` would have to say whether it meant both or either, and
///    it is not left to be supposed. It does not decode.
/// 3. **Either is two grants.** Grants only add, and none takes from
///    another.
/// 4. **Everything else a `T` holds narrows.** A filter's other
///    members — by creator, by template, by what is running — may
///    stand beside either form or alone, and all must hold.
///
/// [`mixed`](Only::mixed) is why a value breaks rule 2, or `None`
/// where it does not. A `T` with no such members — a list of tags to
/// touch — keeps the default.
pub trait Only {
    /// Why this may not be a `within`, by rule 2; `None` when it may.
    fn mixed(&self) -> Option<&'static str> {
        None
    }
}

impl Only for Vec<String> {}

/// Rule 2 of [`Only`], for a filter whose record-naming members are
/// `naming` — each whether it is given — and whose tags are given or
/// not.
pub fn mixed(naming: &[bool], tags: bool) -> Option<&'static str> {
    let named = naming.iter().filter(|given| **given).count();
    if named > 1 {
        return Some("a `within` names records by more than one member: name them by one, and write a grant for each other");
    }
    if named == 1 && tags {
        return Some("a `within` names records and tags together: a grant reaches exactly the records it names, or by tags, never both; write one grant for each");
    }
    None
}

/// The two shapes as a schema: the one word, or `T`'s own. Written by
/// hand because the derive follows a derived serde and this type's
/// serde is its own.
impl<T: JsonSchema> JsonSchema for Within<T> {
    fn schema_name() -> Cow<'static, str> {
        format!("Within_{}", T::schema_name()).into()
    }

    fn schema_id() -> Cow<'static, str> {
        format!("diverge_sdk::shared::permission::Within<{}>", T::schema_id()).into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        let only = generator.subschema_for::<T>();
        json_schema!({
            "anyOf": [
                { "type": "string", "const": "any" },
                only
            ]
        })
    }
}
