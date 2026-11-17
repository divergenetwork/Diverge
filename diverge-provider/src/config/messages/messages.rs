//! The rate messages are relayed at.

use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};

/// The provider relays direct messages between the identities
/// connected to it, at most `messages` sends per sending identity in a
/// fixed window of `seconds`. The rules:
///
/// 1. **Both there, by the document or by default.** A member the
///    section leaves out is its default — thirty messages, sixty
///    seconds — so `messages: {}` is a provider that relays at that
///    rate. There is no unlimited provider: the section absent is no
///    relaying at all, and the defaults are the section's, holding
///    nowhere without it.
/// 2. **Neither zero.** Both are at least `1`; a `0` is refused when
///    the file is read.
/// 3. **Held out as is.** The two are the provider's
///    [`MessagesCapabilities`](diverge_sdk::provider::endpoints::info::server::response::MessagesCapabilities),
///    answered by `info` and enforced by every send.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Messages {
    /// The most sends one identity makes in one window. Absent,
    /// thirty.
    pub messages: NonZeroU32,
    /// How long one window is, in seconds. Absent, sixty.
    pub seconds: NonZeroU32,
}

/// The sends of one window where the document names none: thirty.
const MESSAGES: NonZeroU32 = match NonZeroU32::new(30) {
    Some(messages) => messages,
    None => NonZeroU32::MIN,
};

/// The seconds of one window where the document names none: sixty.
const SECONDS: NonZeroU32 = match NonZeroU32::new(60) {
    Some(seconds) => seconds,
    None => NonZeroU32::MIN,
};

/// What a `messages` section that names nothing gives: thirty sends
/// per sending identity in every sixty seconds, one every two seconds
/// over time and thirty at once.
impl Default for Messages {
    fn default() -> Self {
        Messages { messages: MESSAGES, seconds: SECONDS }
    }
}
