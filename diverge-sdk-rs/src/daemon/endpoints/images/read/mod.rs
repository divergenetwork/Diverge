//! Whether the daemon can supply an image, and, where it can,
//! everything read of it on one scope — the OCI asks, and the image's
//! DIVERGE.md: the provider's
//! [`images::read`](crate::provider::endpoints::images::read), asked of
//! the daemon.
//!
//! The scope's request is the question: an
//! [`Image`](crate::shared::containers::request::Image) — its digest,
//! and the references under which its bytes may be fetched — and the
//! daemons the daemon must not ask for it. The first response answers
//! it. Where the answer is held, the scope stays open, and the caller
//! asks on channels what a provider's read scope answers: a manifest by
//! digest, a blob by digest, in pieces, the proof over the image,
//! the image's DIVERGE.md, parsed, and each of its three icons. The channel requests and their
//! answers are those of the provider's scope, byte for byte.
//!
//! # The rules
//!
//! 1. **The daemon holds nothing.** It answers from a source it asks:
//!    on the request, it opens a read scope with the image on every
//!    connected provider whose capabilities keep a Store or list a
//!    registry, and this scope on every remote daemon it records, all
//!    at once.
//! 2. **Only what the grants reach.** A provider is asked only where a
//!    `providers` grant reaches it with `read`, and a remote daemon only
//!    where a `daemons` grant reaches its record with `relay`, the one
//!    permission to use a remote daemon; see
//!    [`grant`](crate::daemon::grant). An account that holds neither is
//!    answered
//!    [`Forbidden`](server::response::Frame::Forbidden), and nothing is
//!    asked.
//! 3. **Never one it is told to ignore.** A remote daemon whose
//!    identity is in the request's `ignore`, or is this daemon's own,
//!    is not asked. Asking a remote daemon, the daemon sends `ignore`
//!    with its own identity added, so a ring of daemons that record
//!    each other ends.
//! 4. **Found platform by platform.** The request names the platforms
//!    wanted. Each source that answers held says which platforms it
//!    can supply; those wanted that no source before it supplied are
//!    sent in one [`Held`](server::response::Frame::Held), and the
//!    source is kept. A source that adds none is cancelled at once.
//!    Every other answer from a source — not found, unsupported,
//!    forbidden, an error, a scope that would not open — is a no. See
//!    [`Frame`](server::response::Frame) for the stream.
//! 4a. **No longer than it takes.** The moment every platform wanted
//!    is found, every source still being asked is cancelled, unwaited,
//!    and every remote daemon kept is told
//!    [`Enough`](client::channel_request::Frame::Enough), so that it
//!    cancels its own in turn; the scope is
//!    [`Settled`](server::response::Frame::Settled). It is settled too
//!    once the last source has answered, with whatever was found, and
//!    when the caller says `Enough`. With no platform found, the
//!    answer is [`NotFound`](server::response::Frame::NotFound), after
//!    the last source has answered. A request that names no platform
//!    wants any: the first source that can supply anything settles it.
//! 5. **Exactly one answer, first.** Exactly one of `Held`, `NotFound`,
//!    `Forbidden` and [`Error`](server::response::Frame::Error) is the
//!    first response on the main stream. After any but `Held` the
//!    finish follows at once, and no channel is answered.
//! 6. **Served through the sources kept.** Every ask on a held scope
//!    is asked of a source kept, on that source's own scope, and
//!    answered as it answered. A manifest is asked of every source
//!    kept at once, and the first to supply it does. A blob and a
//!    proof are asked first of the source that supplied the manifest
//!    naming them, and of the others in turn where it does not hold
//!    them. The daemon hashes nothing and keeps nothing; an ask no
//!    source answers is answered with the empty finish.
//! 6a. **The document, from the record first.** The one ask the daemon
//!    may answer itself: an image's DIVERGE.md it has read before is
//!    on its record by the image's digest, and answers the `info`
//!    channel with nobody asked. Otherwise the source is asked on its
//!    own `info` channel, the answer relayed, and a document found is
//!    put on the record. It is the one way to an image's document: no
//!    container is asked for it, running or not.
//! 7. **The caller ends it.** After `Settled`, nothing more is sent on
//!    the main stream until the finish, and the finish follows the
//!    caller's [`Cancel`](client::channel_request::Frame::Cancel), or
//!    the caller gone. Every source's scope is cancelled with it. A
//!    source that ends before then is asked nothing more, and what it
//!    alone held is answered with the empty finish.
//!
//! Split by who SENDS, as everywhere else: the request, the channels
//! and the executor are in [`client`]; the answers in [`server`].

pub mod client;
pub mod server;
