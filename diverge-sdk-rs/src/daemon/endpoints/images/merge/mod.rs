//! Merging images into one, in the daemon's Store on a provider. One
//! request, a stream of answers — each blob pulled, each set of
//! platforms added, and the image made.
//!
//! The provider's
//! [`images::merge`](crate::provider::endpoints::images::merge), asked
//! of the daemon: a client names a provider and one or more images,
//! each with the platforms taken of it, and the daemon brings onto
//! that provider whatever of them its Store there lacks, from wherever
//! it finds it, and has the provider make one image of them. The
//! rules:
//!
//! 1. **Judged by the provider.** The caller holds `merge` over
//!    images reaching the provider named, or the answer is
//!    `Forbidden`. `NotFound` for a provider the daemon keeps no
//!    record of; `Unsupported` for one that keeps no Store.
//! 2. **The platforms of each image** are those its entry names, or,
//!    where it names none, every platform the image's manifest names.
//!    A platform two entries take is `PlatformRepeated`, and nothing
//!    moves: one image holds a platform once.
//! 3. **Found wherever they are.** The platforms of an entry the
//!    Store on the provider does not hold are brought in as a push
//!    whose source is the daemon brings them: every provider
//!    connected, and every remote daemon on record, is asked at once
//!    which it can supply, and what supplies a platform is used. Every
//!    entry at once. An entry the Store holds whole is not asked
//!    about. Each blob pulled is one `Blob`, each set of platforms
//!    that lands one `Added`.
//! 4. **`Unavailable` is the one failure of finding.** An image, or a
//!    platform taken of it, that no source supplies ends the merge
//!    `Unavailable`. What landed before it stays in the Store.
//! 5. **Merged.** The provider makes one image of the platforms
//!    taken, kept as the request's description: `Merged`, with its
//!    digest, which is the same for the same entries on every
//!    provider. An image of that digest the daemon owned there already
//!    is `Merged` with `created` false, and is kept as it was.
//! 6. **One whole image is itself.** A request of one entry that
//!    takes every platform its image has, has nothing to join: the
//!    image is brought in, and `Merged` names its own digest.
//! 7. **What else ends it** is what moving bytes can: the Store with
//!    no room, a source that did not prove it holds what it supplied,
//!    bytes that are no image, a platform named that the image has
//!    not.
//!
//! Split by who SENDS, as everywhere else: the request and the
//! executor are in [`client`]; the answers in [`server`].

pub mod client;
pub mod server;
