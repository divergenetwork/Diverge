//! What an icon's file must be.
//!
//! One function per kind of file, used by whoever reads an icon out of
//! an image, so that every party agrees on what an icon is. Each is
//! called on a file already held to
//! [`MAX_BYTES`](crate::shared::containers::info::MAX_BYTES).

use std::fmt;
use std::io::Cursor;

/// The namespace an SVG image's elements are in.
const SVG: &str = "http://www.w3.org/2000/svg";

/// Why a file is not the icon its name says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Invalid {
    /// The file is not UTF-8, which an SVG icon is.
    Utf8,
    /// The file is not well-formed XML, or has a document type
    /// declaration. The message is the parser's.
    Xml(String),
    /// The outermost element is not `svg` in the SVG namespace.
    Root,
    /// The image has a `script` element, or an attribute whose name
    /// begins with `on`.
    Script,
    /// The file is not a PNG image that decodes. The message is the
    /// decoder's.
    Png(String),
    /// The PNG image is not of the size its name says: what it is.
    Size {
        /// Its width in pixels.
        width: u32,
        /// Its height in pixels.
        height: u32,
    },
}

impl fmt::Display for Invalid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Invalid::Utf8 => f.write_str("the icon is not UTF-8"),
            Invalid::Xml(error) => write!(f, "the icon is not well-formed XML with no document type declaration: {error}"),
            Invalid::Root => f.write_str("the icon's outermost element is not `svg` in the SVG namespace"),
            Invalid::Script => f.write_str("the icon has a `script` element or an attribute whose name begins with `on`"),
            Invalid::Png(error) => write!(f, "the icon is not a PNG image: {error}"),
            Invalid::Size { width, height } => write!(f, "the icon is {width} by {height} pixels"),
        }
    }
}

impl std::error::Error for Invalid {}

/// Whether `bytes` are a valid SVG icon. The rules, in the order they
/// are checked:
///
/// 1. The bytes are UTF-8. Otherwise [`Invalid::Utf8`].
/// 2. The text is one well-formed XML document, with no document type
///    declaration: a `<!DOCTYPE` is refused, whatever it declares.
///    Otherwise [`Invalid::Xml`].
/// 3. The document's outermost element is named `svg` and is in the
///    namespace `http://www.w3.org/2000/svg`. Otherwise
///    [`Invalid::Root`].
/// 4. No element of the document, in whatever namespace, is named
///    `script`, compared without regard to ASCII case; and no
///    attribute of any element has a name that begins with `on`,
///    compared without regard to ASCII case. Otherwise
///    [`Invalid::Script`].
///
/// Nothing else is checked: not what the image draws, and not what it
/// refers to.
pub fn svg(bytes: &[u8]) -> Result<(), Invalid> {
    // Rule 1.
    let text = std::str::from_utf8(bytes).map_err(|_| Invalid::Utf8)?;
    // Rule 2: the parser refuses a document type declaration where it
    // is not told to allow one.
    let options = roxmltree::ParsingOptions {
        allow_dtd: false,
        ..roxmltree::ParsingOptions::default()
    };
    let document = roxmltree::Document::parse_with_options(text, options).map_err(|error| Invalid::Xml(error.to_string()))?;
    // Rule 3.
    let root = document.root_element();
    if root.tag_name().name() != "svg" || root.tag_name().namespace() != Some(SVG) {
        return Err(Invalid::Root);
    }
    // Rule 4.
    for node in document.descendants().filter(|node| node.is_element()) {
        if node.tag_name().name().eq_ignore_ascii_case("script") {
            return Err(Invalid::Script);
        }
        if node.attributes().any(|attribute| attribute.name().get(..2).is_some_and(|start| start.eq_ignore_ascii_case("on"))) {
            return Err(Invalid::Script);
        }
    }
    Ok(())
}

/// Whether `bytes` are a valid PNG icon of `side` pixels by `side`.
/// The rules, in the order they are checked:
///
/// 1. The bytes begin as a PNG image does: its signature, and a first
///    chunk that is its header. Otherwise [`Invalid::Png`].
/// 2. The header's width and height are each exactly `side`.
///    Otherwise [`Invalid::Size`]. This is read before any pixel is,
///    so no image larger than the icon is ever decoded.
/// 3. The image decodes: every chunk up to the end of the image's
///    first frame is well-formed, its checksum is right, and the
///    image's data inflates to exactly the pixels the header states.
///    Otherwise [`Invalid::Png`].
///
/// Nothing else is checked: not what the image shows, not its colour
/// type or bit depth, and not what follows its first frame.
pub fn png(bytes: &[u8], side: u32) -> Result<(), Invalid> {
    let invalid = |error: png::DecodingError| Invalid::Png(error.to_string());
    // Rule 1.
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::IDENTITY);
    let mut reader = decoder.read_info().map_err(invalid)?;
    // Rule 2.
    let (width, height) = (reader.info().width, reader.info().height);
    if width != side || height != side {
        return Err(Invalid::Size { width, height });
    }
    // Rule 3.
    let mut pixels = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut pixels).map_err(invalid)?;
    Ok(())
}
