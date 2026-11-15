//! The icons an image's document names.

use serde::{Deserialize, Serialize};

/// The icons an image ships, as its document's `icons` member names
/// them: each member the name of one file in the image's `/.diverge`,
/// beside the document. Every member is optional, and an image may
/// ship any of them, all, or none.
///
/// | member | the file is |
/// |--------|-------------|
/// | `svg`  | an SVG image |
/// | `512`  | a PNG image, 512 pixels wide and 512 high |
/// | `64`   | a PNG image, 64 pixels wide and 64 high |
///
/// What a file holds is its author's word: nothing that carries an
/// icon reads it, so a file that is not what its member says is
/// carried as it is, and its reader finds out. What is checked is the
/// name — see [`name_ok`] — and the file's size, which
/// [`MAX_BYTES`](super::MAX_BYTES) bounds.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Icons {
    /// The SVG icon's file. Absent, the image ships none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub svg: Option<String>,
    /// The 512 by 512 PNG icon's file: the member `512`. Absent, the
    /// image ships none.
    #[serde(default, rename = "512", skip_serializing_if = "Option::is_none")]
    pub png_512: Option<String>,
    /// The 64 by 64 PNG icon's file: the member `64`. Absent, the
    /// image ships none.
    #[serde(default, rename = "64", skip_serializing_if = "Option::is_none")]
    pub png_64: Option<String>,
}

impl Icons {
    /// Whether no icon is named, which a frontmatter leaves out.
    pub fn is_empty(&self) -> bool {
        self.svg.is_none() && self.png_512.is_none() && self.png_64.is_none()
    }

    /// Every name given, in the members' order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        [&self.svg, &self.png_512, &self.png_64].into_iter().filter_map(|name| name.as_deref())
    }
}

/// Whether `name` may name an icon's file. The rules:
///
/// 1. It is 1 to 255 bytes long.
/// 2. Every byte is an ASCII letter, an ASCII digit, `.`, `-` or `_`.
///    So it holds no `/` and no `\`, and names one file of the
///    directory and nothing beneath or above it.
/// 3. Its first byte is not `.`. So it is neither `.` nor `..`.
/// 4. It is not `DIVERGE.md`, compared byte for byte: that file is the
///    document.
pub fn name_ok(name: &str) -> bool {
    (1..=255).contains(&name.len())
        && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
        && !name.starts_with('.')
        && name != "DIVERGE.md"
}
