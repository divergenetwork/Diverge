//! Requests: a question one party puts to others, and the one answer
//! it gets.
//!
//! A party [`create`]s a request — content, as a chat message's is:
//! conversation, not data — and says whom it
//! is made of: accounts and tools, each by name or by tag, or all the
//! party may ask. Every one of them that [`listen`]s is sent it, and
//! any of them may answer. The rules of the whole:
//!
//! 1. **One answer.** The first listener whose answer the daemon
//!    takes has answered the request, and the creator is sent that
//!    answer with who gave it. Every other listener that was sent the
//!    request is told it is withdrawn, and an answer from any of them
//!    is not delivered. Two answers given at once are one taken and
//!    one not.
//! 2. **A request lives with its creator's scope.** The creator
//!    cancelling, or its connection ending, withdraws the request from
//!    everyone it was sent to. The daemon keeps no request past the
//!    scope that made it, and none across a restart.
//! 3. **A listener hears what is made of it, whenever it listens.** A
//!    request made before a listener's scope opened is sent to it when
//!    it opens, where the request is still unanswered; its withdrawal
//!    follows where it comes.
//! 4. **Once each.** A party has at most one listen open, so a request
//!    reaches a party once, and the creator is told of each delivery
//!    once.
//! 5. **Who may be asked** is the creator's grants': the `request`
//!    action over accounts, over standalone tools and over dependency
//!    tools, each with its own reach. A request is made of those it
//!    addresses that the grants reach, and is refused only where that
//!    is nobody. See
//!    [`grant`](crate::daemon::grant). Listening takes no grant: a
//!    listener hears only what was made of it.
//!
//! An account and a tool — standalone or dependency — each listen and
//! each create. A connected standalone tool is another daemon's, has
//! no connection of its own to this one, and does neither.

pub mod create;
pub mod listen;

mod id;

pub use id::*;
