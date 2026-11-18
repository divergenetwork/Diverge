//! What a server's response frame carries for an image publish.

use std::fmt;

use super::{Blob, Published};
use crate::shared::error::Error;
use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};

/// A blob uploaded; the image published; a refusal; or the news that
/// the provider could not.
///
/// A publish is a stream with an end. Zero or more
/// [`Blob`](Self::Blob) frames, one per blob the registry took, as
/// each lands; then exactly one of [`Published`](Self::Published),
/// [`NotFound`](Self::NotFound), [`Incomplete`](Self::Incomplete),
/// [`RegistryUnlisted`](Self::RegistryUnlisted),
/// [`Malformed`](Self::Malformed), [`Unauthorized`](Self::Unauthorized),
/// [`Denied`](Self::Denied), [`Unsupported`](Self::Unsupported) and
/// [`Error`](Self::Error); then the finish, and nothing after it. A
/// payload leads with one byte saying which — `0` for `Blob`, `1` for
/// `Published`, `2` for `NotFound`, `3` for `Incomplete`, `4` for
/// `RegistryUnlisted`, `5` for `Malformed`, `6` for `Unauthorized`,
/// `7` for `Denied`, `8` for `Unsupported`, `9` for `Error` — and the
/// rest is that variant's own bytes; the bare refusals carry nothing.
///
/// | the scope ends with | means |
/// |----------------------|-------|
/// | [`Published`](Self::Published), then a finish | the registry holds the image under the repository, with every tag asked for, and the Store's record of it has the reference |
/// | [`NotFound`](Self::NotFound), then a finish | the caller owns no image by the digest, and nothing was sent |
/// | [`Incomplete`](Self::Incomplete), then a finish | the image does not hold every platform its manifest names, and nothing was sent |
/// | [`RegistryUnlisted`](Self::RegistryUnlisted), then a finish | the registry is none of the provider's, and nothing was sent |
/// | [`Malformed`](Self::Malformed), then a finish | the repository path or a tag is not one, and nothing was sent |
/// | [`Unauthorized`](Self::Unauthorized), then a finish | the registry did not take the credential |
/// | [`Denied`](Self::Denied), then a finish | the registry took the credential, and it may not write to the repository |
/// | [`Unsupported`](Self::Unsupported), then a finish | the provider has no Store |
/// | an [`Error`](Self::Error), then a finish | it is not published, for some other reason |
///
/// # The refusals are answers, not errors
///
/// Each is something a caller acts on: push the platforms missing,
/// name a registry the provider lists, give another credential. A
/// publish that ends with any but `Published` may have left blobs in
/// the registry, which the registry keeps or collects as it does any
/// upload nothing names; no tag was moved unless the answer is
/// `Published`.
#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    /// A blob the registry took from the provider. Tag `0`.
    Blob(Blob),
    /// The registry holds the image, with every tag asked for. Tag
    /// `1`.
    Published(Published),
    /// The caller owns no image by the digest, and nothing was sent.
    /// Tag `2`.
    NotFound,
    /// The image does not hold every platform its manifest names, and
    /// nothing was sent. Tag `3`.
    ///
    /// Carries nothing. A caller pushes the platforms missing — a
    /// listing shows which — and asks again.
    Incomplete,
    /// The registry named is none of the provider's
    /// `images.registries`, and nothing was sent. Tag `4`.
    RegistryUnlisted,
    /// The repository path is not one, or a tag is not one, and
    /// nothing was sent. Tag `5`.
    Malformed,
    /// The registry did not take the credential, or wanted one and
    /// was given none. Tag `6`.
    Unauthorized,
    /// The registry took the credential, and it may not write to the
    /// repository named. Tag `7`.
    Denied,
    /// The provider has no Store — its `images.store` is absent, see
    /// [`info`](crate::provider::endpoints::info) — and nothing was
    /// sent. Tag `8`.
    ///
    /// Carries nothing. A caller asks another provider; nothing about
    /// the request is the cause.
    Unsupported,
    /// A failure. Tag `9`.
    ///
    /// See [`shared::error::Error`](crate::shared::error::Error) for
    /// why it says so little.
    Error(Error),
}

/// Tag for [`Frame::Blob`].
const BLOB: u8 = 0;

/// Tag for [`Frame::Published`].
const PUBLISHED: u8 = 1;

/// Tag for [`Frame::NotFound`].
const NOT_FOUND: u8 = 2;

/// Tag for [`Frame::Incomplete`].
const INCOMPLETE: u8 = 3;

/// Tag for [`Frame::RegistryUnlisted`].
const REGISTRY_UNLISTED: u8 = 4;

/// Tag for [`Frame::Malformed`].
const MALFORMED: u8 = 5;

/// Tag for [`Frame::Unauthorized`].
const UNAUTHORIZED: u8 = 6;

/// Tag for [`Frame::Denied`].
const DENIED: u8 = 7;

/// Tag for [`Frame::Unsupported`].
const UNSUPPORTED: u8 = 8;

/// Tag for [`Frame::Error`].
const ERROR: u8 = 9;

/// Two formats, and the tag chooses between them: a blob and the
/// image published are postcard, as the rest of the Store is, and the
/// error is JSON.
impl Encode for Frame {
    /// One failure per half, and they are different libraries'.
    type Error = FrameEncodeError;

    // Spelled out rather than `Self::Error`: this enum has a variant
    // called `Error`, so the associated type is ambiguous by that name.
    fn encode(&self, out: &mut Writer<'_>) -> Result<(), FrameEncodeError> {
        let bare = |out: &mut Writer<'_>, tag: u8| {
            out.extend_from_slice(&[tag]);
            Ok(())
        };
        match self {
            Frame::Blob(blob) => {
                out.extend_from_slice(&[BLOB]);
                postcard::to_io(blob, &mut *out).map(|_| ()).map_err(FrameEncodeError::Blob)
            }
            Frame::Published(published) => {
                out.extend_from_slice(&[PUBLISHED]);
                postcard::to_io(published, &mut *out).map(|_| ()).map_err(FrameEncodeError::Published)
            }
            Frame::NotFound => bare(out, NOT_FOUND),
            Frame::Incomplete => bare(out, INCOMPLETE),
            Frame::RegistryUnlisted => bare(out, REGISTRY_UNLISTED),
            Frame::Malformed => bare(out, MALFORMED),
            Frame::Unauthorized => bare(out, UNAUTHORIZED),
            Frame::Denied => bare(out, DENIED),
            Frame::Unsupported => bare(out, UNSUPPORTED),
            Frame::Error(error) => {
                out.extend_from_slice(&[ERROR]);
                error.encode(out).map_err(FrameEncodeError::Error)
            }
        }
    }
}

impl Decode<'_> for Frame {
    /// Five ways to fail, and each names which half failed.
    type Error = FrameDecodeError;

    // Spelled out for the same reason as `encode` above.
    fn decode(bytes: &[u8]) -> Result<Self, FrameDecodeError> {
        let (tag, rest) = bytes.split_first().ok_or(FrameDecodeError::Empty)?;
        match *tag {
            BLOB => postcard::from_bytes(rest).map(Frame::Blob).map_err(FrameDecodeError::Blob),
            PUBLISHED => postcard::from_bytes(rest).map(Frame::Published).map_err(FrameDecodeError::Published),
            NOT_FOUND => Ok(Frame::NotFound),
            INCOMPLETE => Ok(Frame::Incomplete),
            REGISTRY_UNLISTED => Ok(Frame::RegistryUnlisted),
            MALFORMED => Ok(Frame::Malformed),
            UNAUTHORIZED => Ok(Frame::Unauthorized),
            DENIED => Ok(Frame::Denied),
            UNSUPPORTED => Ok(Frame::Unsupported),
            ERROR => Error::decode(rest).map(Frame::Error).map_err(FrameDecodeError::Error),
            tag => Err(FrameDecodeError::UnknownTag(tag)),
        }
    }
}

/// An image publish response that could not be written.
#[derive(Debug)]
pub enum FrameEncodeError {
    /// The blob did not serialize.
    Blob(postcard::Error),
    /// The image published did not serialize.
    Published(postcard::Error),
    /// The error did not serialize.
    Error(serde_json::Error),
}

impl fmt::Display for FrameEncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameEncodeError::Blob(error) => write!(f, "image publish blob did not serialize: {error}"),
            FrameEncodeError::Published(error) => write!(f, "image published did not serialize: {error}"),
            FrameEncodeError::Error(error) => write!(f, "image publish error did not serialize: {error}"),
        }
    }
}

impl std::error::Error for FrameEncodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FrameEncodeError::Blob(error) => Some(error),
            FrameEncodeError::Published(error) => Some(error),
            FrameEncodeError::Error(error) => Some(error),
        }
    }
}

/// An image publish response that could not be read.
#[derive(Debug)]
pub enum FrameDecodeError {
    /// No bytes at all, so not even a tag.
    Empty,
    /// A tag that is none of this frame's ten.
    UnknownTag(u8),
    /// The blob did not parse.
    Blob(postcard::Error),
    /// The image published did not parse.
    Published(postcard::Error),
    /// The error did not parse.
    Error(serde_json::Error),
}

impl fmt::Display for FrameDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameDecodeError::Empty => f.write_str("image publish response frame is empty"),
            FrameDecodeError::UnknownTag(tag) => write!(f, "unknown image publish response frame tag {tag}"),
            FrameDecodeError::Blob(error) => write!(f, "image publish blob did not parse: {error}"),
            FrameDecodeError::Published(error) => write!(f, "image published did not parse: {error}"),
            FrameDecodeError::Error(error) => write!(f, "image publish error did not parse: {error}"),
        }
    }
}

impl std::error::Error for FrameDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FrameDecodeError::Blob(error) => Some(error),
            FrameDecodeError::Published(error) => Some(error),
            FrameDecodeError::Error(error) => Some(error),
            FrameDecodeError::Empty | FrameDecodeError::UnknownTag(_) => None,
        }
    }
}
