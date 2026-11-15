//! One of an image's icons, read of the image and answered on the
//! image's own scope.
//!
//! An image's document names the icons it ships — see
//! [`Icons`](crate::shared::containers::info::Icons) — each a file in
//! the image's `/.diverge`. On an
//! [`images::read`](crate::provider::endpoints::images::read) scope
//! that holds the image, a caller opens one channel per icon it wants,
//! each carrying nothing, and the provider answers each with one
//! [`response::Frame`] and the finish. Which icon is which channel:
//! there are three, one per member of `icons`, and no channel takes a
//! name. Nothing is run, and nothing of the image but its document
//! and the file the document names is read.

pub mod response;
