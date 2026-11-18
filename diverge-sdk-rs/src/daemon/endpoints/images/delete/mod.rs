//! Deleting an image in a Store of the daemon's, or some of its
//! platforms. One request, one answer. A client names a provider, a
//! digest, and the platforms to delete; the daemon asks the provider's
//! [`images::delete`](crate::provider::endpoints::images::delete) as
//! itself and answers what came of it, and the scope finishes.
//!
//! The rules:
//!
//! 1. **The daemon's own image.** The daemon is the provider's caller,
//!    so what is deleted is what the daemon owns there: an image it
//!    built or was pushed. Another owner's image of the same digest is
//!    not touched.
//! 2. **No platform named, the whole image.** With `platforms` absent
//!    or empty the image goes. With platforms named, those go and the
//!    rest stay, under the same digest: see the provider's
//!    [request](crate::provider::endpoints::images::delete::client::request::Frame),
//!    whose rules these are.
//! 3. **All of them, or none.** A platform named that the daemon does
//!    not hold of the image there is `PlatformNotHeld`, and nothing is
//!    deleted.
//! 4. **Not found** is a provider the daemon keeps no record of, or a
//!    Store in which the daemon owns no image of the digest.
//! 5. **In use.** An image a running container runs is `InUse`, and
//!    nothing is deleted. The daemon looks at none of its own records
//!    for this: a template that names the digest finds the image again
//!    from another source at its next run, or fails there.
//! 6. **A failure** is a provider that is not connected now, or an
//!    exchange that broke.
//! 7. **Judged by provider.** The caller holds `delete` over images,
//!    reaching the provider named, by its identity or its tags: see
//!    [`grant::images`](crate::daemon::grant::images).
//! 8. **The list follows.** An [`images::list`](super::list) kept open
//!    shows the image gone, or holding fewer platforms, once the
//!    provider has said so.
//!
//! Split by who SENDS, as everywhere else. A client asks — so the
//! question is in [`client`] — and the daemon answers, so the answer is
//! in [`server`]. Neither side holds both halves of the exchange.

pub mod client;
pub mod server;
