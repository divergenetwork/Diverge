//! Which of an image's icons.

/// One of the three icons an image's document may name, as the record
/// keys it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Which {
    /// The SVG icon: `icons.svg`.
    Svg,
    /// The 512 by 512 PNG icon: `icons.512`.
    Png512,
    /// The 64 by 64 PNG icon: `icons.64`.
    Png64,
}

impl Which {
    /// The member of `icons` that names it, which is what the record
    /// keeps it under: `svg`, `512`, `64`.
    pub fn member(self) -> &'static str {
        match self {
            Which::Svg => "svg",
            Which::Png512 => "512",
            Which::Png64 => "64",
        }
    }
}
