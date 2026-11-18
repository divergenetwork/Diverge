//! What a server's response frame carries on the daemon's read scope's
//! main stream.

use serde::Serialize;
use std::fmt;

use crate::shared::error::Error;
use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};

/// Held, or not found, forbidden, or the word that the daemon could
/// not: the answer to the scope's request, which is the question
/// whether the daemon can supply the image.
///
/// Exactly one of `Held`, `NotFound`, `Forbidden` and `Error` is the
/// FIRST response on channel `0`. A payload leads with one byte saying
/// which — `0` for [`Held`](Self::Held), `1` for
/// [`NotFound`](Self::NotFound), `2` for
/// [`Forbidden`](Self::Forbidden), `3` for [`Error`](Self::Error), `4`
/// for [`Settled`](Self::Settled) — and the rest is the platforms'
/// JSON for `Held`, the error's JSON for `Error`, and nothing for the
/// other three.
///
/// # A held scope's main stream
///
/// One or more `Held`, then exactly one `Settled`, then nothing until
/// the finish. The rules:
///
/// 1. **Each held is what one source added.** The daemon asks every
///    source at once. As each answers, the platforms it can supply
///    that the request wants and that no earlier `Held` on this scope
///    carried are sent in one `Held`; a source that adds none is let
///    go and sends nothing. So with `a`, `b` and `c` wanted, a source
///    holding `a` and `b` and a later one holding `b` and `c` are
///    `Held` of `a` and `b`, then `Held` of `c`.
/// 2. **Never a platform not wanted, never one twice.** The platforms
///    of every `Held` on a scope are distinct, and each is one the
///    request named. A request that named none wants any: its one
///    `Held` carries every platform the first source can supply.
/// 3. **Settled, once.** The daemon stops looking when every platform
///    wanted has been sent, when every source has answered, or when
///    the caller says [`Enough`](crate::provider::endpoints::images::read::client::channel_request::Frame::Enough);
///    it then sends `Settled`, and sends no `Held` after it.
/// 4. **Asked on from the first held.** The scope answers channels as
///    soon as its first `Held` is sent, through the sources kept so
///    far.
///
/// | the scope | means |
/// |-----------|-------|
/// | [`Held`](Self::Held), and stays open | a source the daemon asked can supply these platforms of the image, and the daemon answers every ask the caller opens on its channel through the sources it keeps |
/// | [`NotFound`](Self::NotFound), then a finish | no source it asked can supply any platform wanted; no ask is answered |
/// | [`Forbidden`](Self::Forbidden), then a finish | the account holds neither `read` over a provider nor `relay` over a daemon; nothing was asked |
/// | an [`Error`](Self::Error), then a finish | it could not look; no ask is answered |
///
/// # As JSON
///
/// For a client that prints what the daemon answered: the variant,
/// snake case — `{"held":{"platforms":[…]}}`, `"settled"`,
/// `{"error":…}`. The wire is the tag byte and the payload, never
/// this.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Frame {
    /// A source can supply these platforms of the image, and the
    /// daemon serves the scope through it. Tag `0`, then the platforms
    /// as a JSON array of strings.
    Held {
        /// The platforms this source added: wanted, and carried by no
        /// earlier held on the scope.
        platforms: Vec<String>,
    },
    /// No source the daemon asked can. Tag `1`.
    NotFound,
    /// The account the request is served for holds no grant allowing
    /// it; nothing was asked. Tag `2`.
    Forbidden,
    /// A failure. Tag `3`.
    ///
    /// See [`shared::error::Error`](crate::shared::error::Error) for
    /// why it says so little.
    Error(Error),
    /// The daemon looks no further, and no held follows. Tag `4`.
    Settled,
}

/// Tag for [`Frame::Held`].
const HELD: u8 = 0;

/// Tag for [`Frame::NotFound`].
const NOT_FOUND: u8 = 1;

/// Tag for [`Frame::Forbidden`].
const FORBIDDEN: u8 = 2;

/// Tag for [`Frame::Error`].
const ERROR: u8 = 3;

/// Tag for [`Frame::Settled`].
const SETTLED: u8 = 4;

/// A tag, then the error's JSON, or nothing.
impl Encode for Frame {
    /// The ordinary JSON failure, from the one variant that has one.
    /// The tag cannot fail.
    type Error = serde_json::Error;

    // Spelled out rather than `Self::Error`: this enum has a variant
    // called `Error`, so the associated type is ambiguous by that name.
    fn encode(&self, out: &mut Writer<'_>) -> Result<(), serde_json::Error> {
        match self {
            Frame::Held { platforms } => {
                out.extend_from_slice(&[HELD]);
                serde_json::to_writer(out, platforms)
            }
            Frame::Settled => {
                out.extend_from_slice(&[SETTLED]);
                Ok(())
            }
            Frame::NotFound => {
                out.extend_from_slice(&[NOT_FOUND]);
                Ok(())
            }
            Frame::Forbidden => {
                out.extend_from_slice(&[FORBIDDEN]);
                Ok(())
            }
            Frame::Error(error) => {
                out.extend_from_slice(&[ERROR]);
                error.encode(out)
            }
        }
    }
}

impl Decode<'_> for Frame {
    /// Three ways to fail, and only one of them is a parse.
    type Error = FrameError;

    // Spelled out for the same reason as `encode` above.
    fn decode(bytes: &[u8]) -> Result<Self, FrameError> {
        let (tag, rest) = bytes.split_first().ok_or(FrameError::Empty)?;
        match *tag {
            HELD => serde_json::from_slice(rest).map(|platforms| Frame::Held { platforms }).map_err(FrameError::Platforms),
            SETTLED => Ok(Frame::Settled),
            NOT_FOUND => Ok(Frame::NotFound),
            FORBIDDEN => Ok(Frame::Forbidden),
            ERROR => Error::decode(rest).map(Frame::Error).map_err(FrameError::Error),
            tag => Err(FrameError::UnknownTag(tag)),
        }
    }
}

/// An images read response frame that could not be read.
#[derive(Debug)]
pub enum FrameError {
    /// No bytes at all, so not even a tag.
    Empty,
    /// A tag that is none of this frame's five.
    UnknownTag(u8),
    /// The error did not parse.
    Error(serde_json::Error),
    /// The platforms of a held did not parse.
    Platforms(serde_json::Error),
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameError::Empty => f.write_str("images read response frame is empty"),
            FrameError::UnknownTag(tag) => write!(f, "unknown images read response frame tag {tag}"),
            FrameError::Error(error) => write!(f, "images read error did not parse: {error}"),
            FrameError::Platforms(error) => write!(f, "images read platforms did not parse: {error}"),
        }
    }
}

impl std::error::Error for FrameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FrameError::Error(error) | FrameError::Platforms(error) => Some(error),
            FrameError::Empty | FrameError::UnknownTag(_) => None,
        }
    }
}
