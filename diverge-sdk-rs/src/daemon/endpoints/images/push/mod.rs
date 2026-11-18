//! Pushing an image into the daemon's Store on a provider, without
//! building it. One request, a stream of answers, and the channels the
//! daemon opens to ask for the image.
//!
//! The provider's
//! [`images::push`](crate::provider::endpoints::images::push), asked
//! of the daemon: a client names a provider and an image — its digest,
//! and the references under which its bytes may be fetched — and where
//! its bytes come from, and a description if it has one, and the daemon asks that provider's push
//! as itself. The rules:
//!
//! 1. **Into the daemon's Store.** The daemon is the provider's
//!    caller, so what is pushed the daemon owns there, as it owns what
//!    an [`images::build`](super::build) builds: every container the
//!    daemon runs on that provider naming the digest runs it from the
//!    Store, and an [`images::list`](super::list) shows it.
//! 2. **Where the image comes from is the provider's order.** A digest
//!    the daemon owns there already is `Exists`, and nothing moves. A
//!    reference naming a registry the provider uses, which serves the
//!    digest, supplies it, and nobody is asked anything. Otherwise
//!    the request's SOURCE supplies it: the client, or whichever
//!    source the daemon finds, as
//!    [`Source`](client::request::Source) states.
//! 3. **The provider's asks are the source's to answer.** Every
//!    channel the provider opens on its push — a manifest, each blob
//!    it does not hold, and, last and always, the proof — is answered
//!    by the source and by nothing else. Where the source is the
//!    client, the daemon opens each on the client's scope, the ask as
//!    the provider wrote it, and carries the client's answer back as
//!    the client wrote it, reading none of it. Where the source is
//!    the daemon, the daemon answers each from the one read scope it
//!    holds on the source it found, as it answers a provider asking
//!    for the image of a container it runs, and opens nothing on the
//!    client's scope.
//! 4. **An ask the source does not answer is not held.** A client that
//!    reads no channel, that goes away, or that ends an answer short;
//!    a daemon that finds nobody holding the image, or whose source
//!    stops answering: each is answered to the provider as the empty
//!    finish, which is what a caller that does not hold the digest
//!    says; the push is then `Unavailable`, or `ProofFailed`, and
//!    nothing is owned.
//! 5. **Judged by provider alone.** The caller holds `push` over
//!    images, reaching the provider named, by its identity or its
//!    tags: see [`grant::images`](crate::daemon::grant::images). The
//!    source is judged by nothing: a client is the source of its own
//!    image, and the daemon asks as itself, as it does for a run.
//! 6. **A push cannot be stopped** from the client's side: the client
//!    opens no channel on it.
//!
//! Split by who SENDS, as everywhere else: the request, the answers to
//! the daemon's asks and the executor are in [`client`]; the asks and
//! the answers in [`server`].

pub mod client;
pub mod server;
