//! Who a fixed volume is listed to.

use std::fmt;

use diverge_sdk::shared::Identity;
use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Who a fixed volume is listed to: every identity, or the ones
/// named. On the document it is the string `all`, or a list of
/// identities:
///
/// ```yaml
/// authorized: all
/// authorized:
///   - issuer: https://auth.example.com
///     subject: acme
/// ```
///
/// The rules:
///
/// 1. **`all`.** Every identity that connects is listed the volume,
///    and may mount it. What a fixed volume that says nothing is.
/// 2. **A list.** Exactly the identities named, each by its issuer
///    and its subject, compared whole. An identity not among them is
///    not listed the volume and is not told that it exists: naming it
///    in a run or a stat is answered as a name that names nothing.
/// 3. **An empty list is nobody.** It is a list like any other, and
///    lists the volume to no identity.
/// 4. **Asked of nothing.** The answer is the document's alone: no
///    program is run and nothing is read to give it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Authorized {
    /// Every identity: `all`.
    #[default]
    All,
    /// These identities, and no other.
    Identities(Vec<Identity>),
}

/// The word [`Authorized::All`] is on the document.
const ALL: &str = "all";

impl Authorized {
    /// Whether it is [`All`](Self::All): what a fixed volume is where
    /// the document says nothing, and so is not written.
    pub fn is_all(&self) -> bool {
        matches!(self, Authorized::All)
    }

    /// Rules 1 to 3: whether `identity` is listed the volume.
    pub fn admits(&self, identity: &Identity) -> bool {
        match self {
            Authorized::All => true,
            Authorized::Identities(identities) => identities.contains(identity),
        }
    }
}

impl Serialize for Authorized {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Authorized::All => serializer.serialize_str(ALL),
            Authorized::Identities(identities) => identities.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Authorized {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Either;

        impl<'de> Visitor<'de> for Either {
            type Value = Authorized;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("the string `all`, or a list of identities")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Authorized, E> {
                match value {
                    ALL => Ok(Authorized::All),
                    other => Err(E::custom(format!("`{other}` is not `all`; name identities as a list"))),
                }
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Authorized, A::Error> {
                let mut identities = Vec::new();
                while let Some(identity) = seq.next_element::<Identity>()? {
                    identities.push(identity);
                }
                Ok(Authorized::Identities(identities))
            }
        }

        deserializer.deserialize_any(Either)
    }
}
