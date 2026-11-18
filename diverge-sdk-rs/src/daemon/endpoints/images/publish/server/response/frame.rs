//! What a server's response frame carries for an image publish.

use serde::Serialize;
use std::fmt;

use super::{Merged, Published, Uploaded};
use crate::daemon::endpoints::images::push::server::response::{Added, Blob};
use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};
use crate::shared::error::Error;

/// What happened, as it happened, or the one answer that ends the
/// publish.
///
/// A publish is a stream with an end. The rules:
///
/// 1. **What happened, in the order it happened**: zero or more
///    [`Blob`](Self::Blob), one per blob pulled onto the provider;
///    zero or more [`Added`](Self::Added), one per group of platforms
///    that landed there; at most one [`Merged`](Self::Merged), where
///    the parts were made one image; zero or more
///    [`Uploaded`](Self::Uploaded), one per blob the registry took.
/// 2. **Then exactly one of the others**, and the finish, and nothing
///    after it. [`Published`](Self::Published) is the one that ends
///    it well.
/// 3. **Judged first.** `Forbidden`, `NotFound`, `NoProvider`,
///    `RegistryUnlisted`, `Unsupported` and `PlatformRepeated` come
///    with nothing before them: nothing was asked of any provider.
/// 4. **What was told stays true.** An answer that ends the publish
///    badly takes back nothing a `Blob`, an `Added` or a `Merged`
///    before it said: what was brought in is in the daemon's Store on
///    the provider still.
///
/// A payload leads with one byte saying which, and the rest is that
/// variant's own JSON; a bare answer carries nothing after its tag.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Frame {
    /// A blob pulled onto the provider and verified. Tag `0`.
    Blob(Blob),
    /// Some of the platforms taken are now held on the provider. Tag
    /// `1`.
    Added(Added),
    /// The parts are one image in the daemon's Store on the provider.
    /// Tag `2`.
    Merged(Merged),
    /// A blob the registry took. Tag `3`.
    Uploaded(Uploaded),
    /// The registry holds the image. Tag `4`.
    Published(Published),
    /// The caller's grants do not allow it: no `publish` over images,
    /// or none reaching the provider named. Tag `5`.
    Forbidden,
    /// The provider named is none the daemon keeps a record of. Tag
    /// `6`.
    NotFound,
    /// No provider was named, and none the caller's grant reaches is
    /// connected, keeps a Store and uses the registry named. Tag `7`.
    NoProvider,
    /// The provider named does not use the registry named. Tag `8`.
    RegistryUnlisted,
    /// The provider named keeps no Store. Tag `9`.
    Unsupported,
    /// A platform taken has no source: no provider and no remote
    /// daemon the daemon reaches supplies it, or a digest named is one
    /// none supplies at all. Tag `10`.
    Unavailable,
    /// Two parts take the same platform, and nothing was asked of any
    /// provider. Tag `11`.
    PlatformRepeated,
    /// A platform a part names is not one its image has. Tag `12`.
    PlatformNotFound,
    /// The provider's Store has no room. Tag `13`.
    InsufficientCapacity,
    /// A source supplied a part and did not prove it holds it. Tag
    /// `14`.
    ProofFailed,
    /// A part is not an image the provider takes, or the repository
    /// path or a tag is not one. Tag `15`.
    Malformed,
    /// The registry did not take the credential. Tag `16`.
    Unauthorized,
    /// The credential may not write to the repository. Tag `17`.
    Denied,
    /// A failure, in the daemon's words or the provider's. Tag `18`.
    Error(Error),
}

/// Tag for [`Frame::Blob`].
const BLOB: u8 = 0;

/// Tag for [`Frame::Added`].
const ADDED: u8 = 1;

/// Tag for [`Frame::Merged`].
const MERGED: u8 = 2;

/// Tag for [`Frame::Uploaded`].
const UPLOADED: u8 = 3;

/// Tag for [`Frame::Published`].
const PUBLISHED: u8 = 4;

/// Tag for [`Frame::Forbidden`].
const FORBIDDEN: u8 = 5;

/// Tag for [`Frame::NotFound`].
const NOT_FOUND: u8 = 6;

/// Tag for [`Frame::NoProvider`].
const NO_PROVIDER: u8 = 7;

/// Tag for [`Frame::RegistryUnlisted`].
const REGISTRY_UNLISTED: u8 = 8;

/// Tag for [`Frame::Unsupported`].
const UNSUPPORTED: u8 = 9;

/// Tag for [`Frame::Unavailable`].
const UNAVAILABLE: u8 = 10;

/// Tag for [`Frame::PlatformRepeated`].
const PLATFORM_REPEATED: u8 = 11;

/// Tag for [`Frame::PlatformNotFound`].
const PLATFORM_NOT_FOUND: u8 = 12;

/// Tag for [`Frame::InsufficientCapacity`].
const INSUFFICIENT_CAPACITY: u8 = 13;

/// Tag for [`Frame::ProofFailed`].
const PROOF_FAILED: u8 = 14;

/// Tag for [`Frame::Malformed`].
const MALFORMED: u8 = 15;

/// Tag for [`Frame::Unauthorized`].
const UNAUTHORIZED: u8 = 16;

/// Tag for [`Frame::Denied`].
const DENIED: u8 = 17;

/// Tag for [`Frame::Error`].
const ERROR: u8 = 18;

/// The tag, then the variant's own JSON; a bare answer carries
/// nothing after its tag.
impl Encode for Frame {
    /// The ordinary JSON failure. The tag cannot fail.
    type Error = serde_json::Error;

    // Spelled out rather than `Self::Error`: this enum has a variant
    // called `Error`, so the associated type is ambiguous by that name.
    fn encode(&self, out: &mut Writer<'_>) -> Result<(), serde_json::Error> {
        let bare = |out: &mut Writer<'_>, tag: u8| {
            out.extend_from_slice(&[tag]);
            Ok(())
        };
        match self {
            Frame::Blob(value) => {
                out.extend_from_slice(&[BLOB]);
                serde_json::to_writer(out, value)
            }
            Frame::Added(value) => {
                out.extend_from_slice(&[ADDED]);
                serde_json::to_writer(out, value)
            }
            Frame::Merged(value) => {
                out.extend_from_slice(&[MERGED]);
                serde_json::to_writer(out, value)
            }
            Frame::Uploaded(value) => {
                out.extend_from_slice(&[UPLOADED]);
                serde_json::to_writer(out, value)
            }
            Frame::Published(value) => {
                out.extend_from_slice(&[PUBLISHED]);
                serde_json::to_writer(out, value)
            }
            Frame::Forbidden => bare(out, FORBIDDEN),
            Frame::NotFound => bare(out, NOT_FOUND),
            Frame::NoProvider => bare(out, NO_PROVIDER),
            Frame::RegistryUnlisted => bare(out, REGISTRY_UNLISTED),
            Frame::Unsupported => bare(out, UNSUPPORTED),
            Frame::Unavailable => bare(out, UNAVAILABLE),
            Frame::PlatformRepeated => bare(out, PLATFORM_REPEATED),
            Frame::PlatformNotFound => bare(out, PLATFORM_NOT_FOUND),
            Frame::InsufficientCapacity => bare(out, INSUFFICIENT_CAPACITY),
            Frame::ProofFailed => bare(out, PROOF_FAILED),
            Frame::Malformed => bare(out, MALFORMED),
            Frame::Unauthorized => bare(out, UNAUTHORIZED),
            Frame::Denied => bare(out, DENIED),
            Frame::Error(error) => {
                out.extend_from_slice(&[ERROR]);
                error.encode(out)
            }
        }
    }
}

impl Decode<'_> for Frame {
    /// Four ways to fail, and two of them are a parse.
    type Error = FrameError;

    // Spelled out for the same reason as `encode` above.
    fn decode(bytes: &[u8]) -> Result<Self, FrameError> {
        let (tag, rest) = bytes.split_first().ok_or(FrameError::Empty)?;
        match *tag {
            BLOB => serde_json::from_slice(rest).map(Frame::Blob).map_err(FrameError::Body),
            ADDED => serde_json::from_slice(rest).map(Frame::Added).map_err(FrameError::Body),
            MERGED => serde_json::from_slice(rest).map(Frame::Merged).map_err(FrameError::Body),
            UPLOADED => serde_json::from_slice(rest).map(Frame::Uploaded).map_err(FrameError::Body),
            PUBLISHED => serde_json::from_slice(rest).map(Frame::Published).map_err(FrameError::Body),
            FORBIDDEN => Ok(Frame::Forbidden),
            NOT_FOUND => Ok(Frame::NotFound),
            NO_PROVIDER => Ok(Frame::NoProvider),
            REGISTRY_UNLISTED => Ok(Frame::RegistryUnlisted),
            UNSUPPORTED => Ok(Frame::Unsupported),
            UNAVAILABLE => Ok(Frame::Unavailable),
            PLATFORM_REPEATED => Ok(Frame::PlatformRepeated),
            PLATFORM_NOT_FOUND => Ok(Frame::PlatformNotFound),
            INSUFFICIENT_CAPACITY => Ok(Frame::InsufficientCapacity),
            PROOF_FAILED => Ok(Frame::ProofFailed),
            MALFORMED => Ok(Frame::Malformed),
            UNAUTHORIZED => Ok(Frame::Unauthorized),
            DENIED => Ok(Frame::Denied),
            ERROR => Error::decode(rest).map(Frame::Error).map_err(FrameError::Error),
            tag => Err(FrameError::UnknownTag(tag)),
        }
    }
}

/// An images publish response frame that could not be read.
#[derive(Debug)]
pub enum FrameError {
    /// No bytes at all, so not even a tag.
    Empty,
    /// A tag that is none of this frame's nineteen.
    UnknownTag(u8),
    /// What the tag promised did not parse.
    Body(serde_json::Error),
    /// The error did not parse.
    Error(serde_json::Error),
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameError::Empty => f.write_str("images publish response frame is empty"),
            FrameError::UnknownTag(tag) => write!(f, "unknown images publish response frame tag {tag}"),
            FrameError::Body(error) => write!(f, "images publish answer did not parse: {error}"),
            FrameError::Error(error) => write!(f, "images publish error did not parse: {error}"),
        }
    }
}

impl std::error::Error for FrameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FrameError::Body(error) | FrameError::Error(error) => Some(error),
            FrameError::Empty | FrameError::UnknownTag(_) => None,
        }
    }
}
