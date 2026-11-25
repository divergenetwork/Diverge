//! One secret an image's program may take.

use serde::{Deserialize, Serialize};

use crate::shared::description::Description;

/// The most characters a secret's name holds: the limit of the `name`
/// of a Chrome extension's manifest, as
/// [`DESCRIPTION_CHARACTERS`](crate::shared::description::DESCRIPTION_CHARACTERS)
/// is the limit of that manifest's `description`.
pub const SECRET_NAME_CHARACTERS: usize = 75;

/// One secret an image's program may take, as the image's document
/// states it: its name, and what it is, in one line.
///
/// An image states every secret its program could ever take, whatever
/// its arguments. Which of them a container of the image needs is its
/// template's to say — the `secrets` of a
/// [daemon's template](crate::daemon::template::Template) or of a
/// [dependency's declaration](crate::shared::containers::dependencies::Template),
/// each a name the image states, all of them or some — since which a
/// program takes follows from the arguments it is given. The
/// description is for whoever makes a template: what the secret is,
/// and so when it is needed.
///
/// The rules of a name, which [`Document::parse`](super::Document::parse)
/// holds every one to:
///
/// 1. **Plain text.** One line: no control character — no newline, no
///    tab — anywhere in it.
/// 2. **Short.** At least one character, and at most
///    [`SECRET_NAME_CHARACTERS`], counted as Unicode scalar values.
/// 3. **Trimmed.** No whitespace at its start or its end.
/// 4. **Its own.** No two secrets of one document have one name.
///
/// The description is a [`Description`], and so is held to that type's
/// own rules wherever it is read.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Secret {
    /// The secret's name: what a template names it by, what a record
    /// binds to a vault key, and what the program declares and is fed
    /// it under.
    pub name: String,
    /// What the secret is, in one line: a [`Description`], and so at
    /// most
    /// [`DESCRIPTION_CHARACTERS`](crate::shared::description::DESCRIPTION_CHARACTERS)
    /// characters of plain text.
    pub description: Description,
}

impl Secret {
    /// Whether `name` is a secret's name by rules 1 to 3. Rule 4 is
    /// of a document's secrets together, and is the document's to
    /// hold.
    pub fn named(name: &str) -> bool {
        let characters = name.chars().count();
        (1..=SECRET_NAME_CHARACTERS).contains(&characters) && !name.chars().any(char::is_control) && name.trim() == name
    }
}
