//! Publishing images to a registry as one image. One request, a
//! stream of answers. A client names one or more images by digest, the
//! platforms to take of each, and a repository at a registry; the
//! daemon picks a provider, brings onto it whatever it lacks, makes
//! one image of the parts where there is more than one, and has the
//! provider send it to the registry.
//!
//! The rules:
//!
//! 1. **The platforms of each part.** A part that names platforms
//!    takes those. A part that names none takes every platform its
//!    manifest names, which the daemon reads from a Store listing it
//!    keeps where one holds the digest, and through the first source
//!    that can supply the image where none does. A digest no source
//!    supplies is `Unavailable`.
//! 2. **Each platform once.** A platform two parts take is
//!    `PlatformRepeated`, and nothing is asked of any provider: an
//!    image has one image per platform.
//! 3. **The provider.** The work is done on one provider: one that is
//!    connected, keeps a Store, lists the registry named among those
//!    it uses, and that the caller's `publish` grant reaches. Named by
//!    the request, it is that one, or the answer saying which of
//!    these it is not. Not named, it is the one whose Store of the
//!    daemon's holds the most of the platforms taken already, the
//!    first on the daemon's record where several hold as many; with no
//!    such provider the answer is `NoProvider`.
//! 4. **Brought in.** Every part's platforms that the provider does
//!    not hold are pushed onto it as an
//!    [`images::push`](super::push) whose source is the daemon pushes
//!    them: a source found for every platform before anything moves,
//!    then one push for each source, all at once, the parts beside
//!    each other. Each blob pulled is a `Blob`, and each group of
//!    platforms that lands an `Added`, sent as it happens.
//! 5. **One image.** A request of one part that takes every platform
//!    its manifest names publishes that image, by its own digest. Any
//!    other is merged first, by the provider's
//!    [`images::merge`](crate::provider::endpoints::images::merge):
//!    one new image, an index over exactly the platforms taken, whose
//!    digest is sent as `Merged`. The same parts make the same digest
//!    on every provider.
//! 6. **Published.** The provider's
//!    [`images::publish`](crate::provider::endpoints::images::publish)
//!    sends the image to the repository, with the tags and the
//!    credential of the request. Each blob the registry takes is an
//!    `Uploaded`. The answer that ends it well is `Published`: the
//!    provider it was done on, the digest the registry holds it by,
//!    and whether the image was made by this request or was in the
//!    daemon's Store there already.
//! 7. **The description is the new image's.** It is what an image this
//!    request makes in the Store is kept as. An image that was there
//!    already keeps what it was kept as, and the description is not
//!    used.
//! 8. **What was brought in stays.** A request that fails part way
//!    leaves in the daemon's Store on the provider what it had pushed
//!    and merged by then, and one that succeeds leaves it all:
//!    [`images::delete`](super::delete) removes it.
//! 9. **The credential is used and not kept.** The daemon hands it to
//!    the provider with the publish and keeps it nowhere; the record
//!    of the request it keeps has no password in it.
//! 10. **Judged by provider.** The caller holds `publish` over images,
//!     reaching the provider the work is done on: see
//!     [`grant::images`](crate::daemon::grant::images). The sources
//!     the parts are brought from are judged by nothing, as a push's
//!     are not.
//! 11. **A publish cannot be stopped** from the client's side: the
//!     client opens no channel on it.
//!
//! Split by who SENDS, as everywhere else: the request and the
//! executor are in [`client`]; the answers in [`server`].

pub mod client;
pub mod server;
