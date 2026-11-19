//! Changing what the daemon says of an image in its Store on a
//! provider: its description. One request, one answer.
//!
//! The provider's
//! [`images::edit`](crate::provider::endpoints::images::edit), asked
//! of the daemon: a client names a provider and an image by its
//! digest, as an [`images::list`](super::list) answers them, and the
//! description the image is kept as from now on. The rules:
//!
//! 1. **Judged by the provider.** The caller holds `edit` over images
//!    reaching the provider named, or the answer is `Forbidden`.
//! 2. **The daemon's own image.** The daemon asks the provider as
//!    itself, so what is edited is the image the daemon owns there.
//!    `NotFound` for a provider the daemon keeps no record of, and for
//!    a digest the daemon owns no image by on it.
//! 3. **The description replaces, and absent clears.** The image's
//!    description becomes the request's, whole. A request with none
//!    leaves the image with none.
//! 4. **Nothing else changes.** An image's bytes and its platforms are
//!    its digest's and its Store's: the description is the one thing
//!    of it a request changes. A listing says `Changed`.
//!
//! Split by who SENDS, as everywhere else: the request and the
//! executor are in [`client`]; the answer in [`server`].

pub mod client;
pub mod server;
