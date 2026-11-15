//! What a deployer read of one file of an image's `/.diverge`.

/// The answer to [`read_file`](super::container_deployer::ContainerDeployer::read_file):
/// four answers, none of them a failure, since a failure is the
/// deployer's own error.
///
/// The bytes are the file's as the image holds them, unread: whether
/// they are text, and a document, is the handler's to judge, with
/// [`Document::parse`](diverge_sdk::shared::containers::info::Document::parse),
/// so a deployer never speaks the document's rules. What a deployer
/// does hold a file to is its size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageInfo {
    /// The file, a regular one in the image's `/.diverge`.
    Found(Vec<u8>),
    /// The image's `/.diverge` holds no file of the name, or what is
    /// at that path is not a regular file.
    Missing,
    /// The file is larger than
    /// [`MAX_BYTES`](diverge_sdk::shared::containers::info::MAX_BYTES):
    /// measured, and not read.
    TooLarge,
    /// The image is in none of the places the deployer looks.
    Unavailable,
}
