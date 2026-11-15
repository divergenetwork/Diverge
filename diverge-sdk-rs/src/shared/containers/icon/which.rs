//! Which of an image's three icons.

use super::valid::{self, Invalid};

/// One of the three icons an image may ship, each a file of its
/// `/.diverge` by a fixed name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Which {
    /// The SVG icon: `/.diverge/icon.svg`.
    Svg,
    /// The 512 by 512 PNG icon: `/.diverge/icon-512.png`.
    Png512,
    /// The 64 by 64 PNG icon: `/.diverge/icon-64.png`.
    Png64,
}

impl Which {
    /// All three, in the order of their channels.
    pub const ALL: [Which; 3] = [Which::Svg, Which::Png512, Which::Png64];

    /// The icon's file name in the image's `/.diverge`: exactly
    /// `icon.svg`, `icon-512.png` or `icon-64.png`.
    pub fn file(self) -> &'static str {
        match self {
            Which::Svg => "icon.svg",
            Which::Png512 => "icon-512.png",
            Which::Png64 => "icon-64.png",
        }
    }

    /// The icon whose file name `file` is, if it is one.
    pub fn of(file: &str) -> Option<Which> {
        Which::ALL.into_iter().find(|which| which.file() == file)
    }

    /// Whether `bytes` are this icon: a valid SVG image, or a valid
    /// PNG image of the size the icon is. See [`valid`].
    pub fn validate(self, bytes: &[u8]) -> Result<(), Invalid> {
        match self {
            Which::Svg => valid::svg(bytes),
            Which::Png512 => valid::png(bytes, 512),
            Which::Png64 => valid::png(bytes, 64),
        }
    }
}
