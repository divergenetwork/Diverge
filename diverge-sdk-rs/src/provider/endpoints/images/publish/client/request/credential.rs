//! What a publish writes to the registry as.

use std::fmt;

use serde::{Deserialize, Serialize};

/// The credential a publish presents to the registry: a username and
/// a password. The rules:
///
/// 1. **Two values, as a registry takes them.** The password is
///    whatever the registry accepts beside the username — a password,
///    an access token, a robot's secret.
/// 2. **For this publish alone.** The provider presents it to the
///    registry named and to that registry's token service, for the
///    length of the publish, and keeps nothing of it after: it is
///    written to no disk, kept in no cache another request could read,
///    and reported by no listing.
/// 3. **Never shown.** [`Debug`] prints the username and no password.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Credential {
    /// The username.
    pub username: String,
    /// The password, or the token the registry takes in its place.
    pub password: String,
}

/// The username, and that there is a password: rule 3.
impl fmt::Debug for Credential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Credential").field("username", &self.username).field("password", &"…").finish()
    }
}
