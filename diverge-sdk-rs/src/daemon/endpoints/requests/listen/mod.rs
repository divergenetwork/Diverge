//! Hearing the requests made of the caller, and answering them. One
//! request, a stream that never ends by itself, and the channels the
//! caller opens to answer and to stop.
//!
//! The rules:
//!
//! 1. **What is heard.** Every request whose creator addressed the
//!    caller and may ask it: each is one
//!    [`Request`](server::response::Frame::Request) on the main
//!    stream, with its id, its content as the creator wrote it, and
//!    who the creator is. The
//!    ones pending when the scope opens come first, in no order of
//!    their own; each later one as it is created, or as the caller
//!    comes to be addressed by it.
//! 2. **Withdrawn.** A request the caller was sent and that is wanted
//!    no more — another listener answered it, its creator cancelled,
//!    or its creator's connection ended — is one
//!    [`Withdrawn`](server::response::Frame::Withdrawn), with its id.
//!    A listener may be sent the withdrawal of an id it was never
//!    sent, and ignores it. The listener whose own answer was taken
//!    is sent none: it knows.
//! 3. **Answered on a channel.** The caller answers a request by
//!    opening [`Respond`](client::channel_request::Frame::Respond)
//!    with the request's id and its content. The daemon answers
//!    that channel with exactly one
//!    [`Delivery`](server::channel_response::Frame): `Delivered`, the
//!    answer was taken and is the request's; or `NotDelivered`, it was
//!    not — the request was answered or withdrawn first, was never
//!    sent to this listener, or is none there is. Then the finish.
//! 4. **One listen for each party.** An account, an agent and a tool has at
//!    most one listen open on the daemon, on whatever connection. A
//!    second is answered
//!    [`AlreadyOpen`](server::response::Frame::AlreadyOpen), alone,
//!    and the first is not disturbed.
//! 5. **It ends when the caller ends it**: by
//!    [`Cancel`](client::channel_request::Frame::Cancel), or by its
//!    connection ending. What it was sent and did not answer stays
//!    pending for whoever else was sent it, and for the caller when it
//!    listens again.
//! 6. **No grant.** Any account, any agent and any tool may listen. An account
//!    that is no longer one is answered `Forbidden`.
//!
//! Split by who SENDS, as everywhere else: the request, the channels
//! and the executor are in [`client`]; the answers in [`server`].

pub mod client;
pub mod server;
