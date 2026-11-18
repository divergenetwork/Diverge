//! What the tool asks of the daemon, declared at its registration.

use diverge_sdk::shared::permission::identities::{Make, Over, Permission};
use diverge_sdk::shared::permission::{Grant, Reach, images};

/// The tool's permissions: to log in, to publish images, and nothing
/// else. The rules:
///
/// 1. **The login that makes.** `{"identities": ["login"]}`: signing
///    in an identity that is not on the daemon's host yet — a subject's
///    first login there.
/// 2. **The login that replaces.** `{"identities": {"actions":
///    ["login"], "within": "any"}}`: signing in again one that is,
///    whatever its tags — a subject signed out, or signed in anew.
/// 3. **The publish.** `{"images": {"actions": ["publish"], "within":
///    "any"}}`: publishing images from whichever provider the daemon
///    picks, or the caller names.
/// 4. **No more.** The tool lists no identity and tags none, and
///    builds, pushes, lists and deletes no image: what is on the host
///    and in the Stores is not its to read, and it asks the daemon
///    for nothing but the login and the publish.
///
/// All three are what `/.diverge/DIVERGE.md` says the tool needs. A
/// daemon that grants the tool no login answers every login
/// `forbidden`, and one that grants it no publish every publish.
pub fn permissions() -> Vec<Grant> {
    vec![
        // Rule 1.
        Grant::Identities(Permission::Make(vec![Make::Login])),
        // Rule 2.
        Grant::Identities(Permission::Over { actions: vec![Over::Login], within: Reach::Any }),
        // Rule 3.
        Grant::Images(images::Permission::Over { actions: vec![images::Action::Publish], within: Reach::Any }),
    ]
}
