//! An image's icons: three files beside its document, by fixed names,
//! each read of the image and answered on the image's own scope.
//!
//! An image keeps what it says of itself in the directory `.diverge`
//! at the root of its filesystem. Its document is one file of it. Its
//! icons are three more, each optional, each by a name that never
//! changes — see [`Which`]:
//!
//! | file | is |
//! |------|----|
//! | `/.diverge/icon.svg` | an SVG image |
//! | `/.diverge/icon-512.png` | a PNG image, 512 pixels wide and 512 high |
//! | `/.diverge/icon-64.png` | a PNG image, 64 pixels wide and 64 high |
//!
//! The rules:
//!
//! 1. **An icon is there when its file is.** No document names it, and
//!    none is read to find it.
//! 2. **An icon is at most
//!    [`MAX_BYTES`](crate::shared::containers::info::MAX_BYTES)**, as
//!    every file of the directory is.
//! 3. **An icon is what its name says.** A file that is not a valid
//!    SVG image, or not a valid PNG image of its name's size, is no
//!    icon: [`valid`] states what each is, and whoever reads the file
//!    out of an image checks it before it answers it.
//!
//! On an [`images::read`](crate::provider::endpoints::images::read)
//! scope that holds the image, a caller opens one channel per icon it
//! wants, each carrying nothing, and the provider answers each with
//! one [`response::Frame`] and the finish. There are three channels,
//! one per icon, and none takes a name. Nothing is run, and nothing of
//! the image but the one file is read.

pub mod response;
pub mod valid;

mod which;

pub use which::*;
