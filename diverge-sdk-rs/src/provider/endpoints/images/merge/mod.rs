//! Making one image of the platforms of several.
//!
//! A caller names images it holds in its Store and, for each, the
//! platforms to take from it; the provider writes one new image, an
//! index over exactly those, and the caller owns it by its digest.
//! Nothing a source image is changes, and no layer moves: the new
//! image reaches what its sources reach. Split by who SENDS, as
//! everywhere else: the ask is in [`client`], the answer in
//! [`server`].

pub mod client;
pub mod server;
