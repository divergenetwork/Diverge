//! Making a request of others, and waiting for its answer. One
//! request, a stream of what becomes of it, and the channel the
//! caller opens to take it back.
//!
//! The rules:
//!
//! 1. **Whom it is made of** is the request's
//!    [`to`](client::request::To): all the caller may ask, or the
//!    accounts and tools it names, each by itself or by a tag.
//! 2. **Judged before it exists.** The caller holds `request` over
//!    accounts, over standalone tools or over dependency tools, or the
//!    answer is [`Forbidden`](server::response::Frame::Forbidden).
//!    Every account and tool named is one there is, or it is
//!    [`NotFound`](server::response::Frame::NotFound); and is one the
//!    caller's `request` grant for its kind reaches, or it is
//!    `Forbidden`. A tag named is one whose every record, as of now,
//!    the grant for its kind reaches, or it is `Forbidden`. `all`
//!    names nothing and is refused for nothing: it is made of exactly
//!    what the grants reach.
//! 3. **Created.** [`Created`](server::response::Frame::Created) is
//!    the first response, with the id the daemon drew for it.
//! 4. **Delivered, once each.** Every listener the request is made of
//!    is sent it: those listening now, at once, and each that comes
//!    to listen after, when it does. Each is one
//!    [`Delivered`](server::response::Frame::Delivered), naming whom,
//!    and no listener is named twice. A listener is judged as it is
//!    sent the request, by the caller's grants as they were at the
//!    create: one it is not made of, or the grants do not reach, is
//!    sent nothing.
//! 5. **Answered.** The first listener whose answer the daemon takes
//!    ends the request:
//!    [`Responded`](server::response::Frame::Responded), the answer
//!    and who gave it, then the finish. Every other listener it was
//!    sent to is told it is withdrawn.
//! 6. **Taken back.** The caller's
//!    [`Cancel`](client::channel_request::Frame::Cancel), or its
//!    connection ending, withdraws the request from every listener it
//!    was sent to, and the scope finishes with no answer. A request
//!    nobody answers is waited on for as long as its scope is open:
//!    the daemon sets no time on it.
//!
//! Split by who SENDS, as everywhere else: the request, the channel
//! and the executor are in [`client`]; the answers in [`server`].

pub mod client;
pub mod server;
