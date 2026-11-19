//! Whether a daemon announces itself on a provider, and with what.

use serde::{Deserialize, Serialize};

use super::description::Description;

/// What a provider's record says of the daemon announcing itself on
/// it: that it does not, that it does, or that it does with a text.
/// Untagged on the wire: a boolean, or a string.
///
/// | written | means |
/// |---------|-------|
/// | `false` | the daemon does not announce itself on the provider |
/// | `true` | it announces itself, with no text |
/// | `"…"` | it announces itself with this text |
///
/// The rules:
///
/// 1. **Announced is listed.** A daemon that announces itself on a
///    provider is in that provider's
///    [`daemons::list`](crate::provider::endpoints::daemons::list) for
///    as long as it is connected, and is what another daemon finds it
///    by. One that does not is found by nobody there.
/// 2. **The text is a [`Description`].** One line of plain text, 1 to
///    132 characters, trimmed. A string that is none — the empty
///    string among them — is no announcement, and does not
///    deserialize: an announcement with no text is `true`, never `""`.
/// 3. **Nothing else.** `null` and a number are no announcement.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(untagged)]
pub enum Announcement {
    /// `false`: not announced. `true`: announced, with no text.
    Whether(bool),
    /// Announced, with this text.
    Text(Description),
}

impl Announcement {
    /// What the daemon holds open on the provider: `None`, no
    /// announce; `Some(None)`, an announce with no text; `Some(Some)`,
    /// an announce with the text.
    pub fn announce(&self) -> Option<Option<&Description>> {
        match self {
            Announcement::Whether(false) => None,
            Announcement::Whether(true) => Some(None),
            Announcement::Text(text) => Some(Some(text)),
        }
    }
}
