//! How large a file of an image's `/.diverge` may be.

/// The most bytes a file read out of an image's `/.diverge` may hold:
/// `1048576`, which is 2 to the power 20. It bounds the file as the
/// image holds it — its length in bytes, before anything is parsed,
/// decoded or decompressed — and it bounds each file by itself: the
/// document, `/.diverge/DIVERGE.md`, and each of the three icons.
///
/// The rules:
///
/// 1. A file of exactly `1048576` bytes is within the limit.
/// 2. A file of `1048577` bytes or more is over it. A provider does
///    not answer such a file's content, whole or in part: it answers
///    that the file is too large, as
///    [`info::response::Frame::TooLarge`](super::response::Frame::TooLarge)
///    and
///    [`icon::response::Frame::TooLarge`](crate::shared::containers::icon::response::Frame::TooLarge)
///    state.
/// 3. The limit is on the file, not on the frame that carries it: a
///    frame's tag byte is not counted, and a document's JSON on the
///    wire, which is the file parsed and written again, is not
///    measured.
pub const MAX_BYTES: u64 = 1_048_576;
