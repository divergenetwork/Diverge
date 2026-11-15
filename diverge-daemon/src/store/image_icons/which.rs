//! Which of an image's icons.

/// One of an image's three icons, as the record keys it.
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
    /// What the record keeps it under: `svg`, `512`, `64`.
    pub fn member(self) -> &'static str {
        match self {
            Which::Svg => "svg",
            Which::Png512 => "512",
            Which::Png64 => "64",
        }
    }
}
