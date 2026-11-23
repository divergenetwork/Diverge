//! A subject of `identity.diverge.network`, read.

use super::Error;

/// The username of the account that owns `subject` at
/// `identity.diverge.network`, and the subject's name:
/// `<username>.<name>`, read. The rules:
///
/// 1. **Split at the first period**, and no other. A username holds
///    none, and the name is everything after it.
/// 2. **Neither is empty.**
///
/// What a username and a name may otherwise be is
/// `identity.diverge.network`'s to say, and is not held to here. A
/// subject that is not so is [`Error::Subject`]: no subject of
/// `identity.diverge.network`'s.
pub fn parts(subject: &str) -> Result<(String, String), Error> {
    match subject.split_once('.') {
        Some((username, name)) if !username.is_empty() && !name.is_empty() => Ok((username.to_string(), name.to_string())),
        _ => Err(Error::Subject(subject.to_string())),
    }
}
