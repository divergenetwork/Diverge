//! What a server's response frame carries for a delete.

use serde::Serialize;
use std::fmt;

use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};
use crate::shared::error::Error;

/// A delete's answer: the compartment is gone, none is the one named,
/// it is not deletable, it is in use, forbidden, or a failure.
///
/// One question and one reply, so there is exactly one of these per
/// scope, before the finish that ends it. A payload leads with one
/// byte saying which — `0` for [`Deleted`](Self::Deleted), `1` for
/// [`NotFound`](Self::NotFound), `2` for
/// [`NotDeletable`](Self::NotDeletable), `3` for
/// [`InUse`](Self::InUse), `4` for [`Forbidden`](Self::Forbidden), `5`
/// for [`Error`](Self::Error) — and only the error carries anything
/// after it.
///
/// [`NotFound`](Self::NotFound), [`NotDeletable`](Self::NotDeletable)
/// and [`InUse`](Self::InUse) are ANSWERS: the daemon looked, nothing
/// changed and nothing is retried. An [`Error`](Self::Error) is the
/// absence of an answer, and the compartment is as it was.
///
/// # Forbidden
///
/// [`Forbidden`](Self::Forbidden) is an answer every endpoint has: the
/// account the request is served for — the connection's — or the tool
/// the request came from holds no grant allowing
/// what the request asks over what it names, and nothing changed. Where
/// the answer is a stream it comes as an error does: exactly one, with
/// nothing before it, then the finish. Nothing is retried. See
/// [`grant`](crate::daemon::grant).
///
/// # As JSON
///
/// For a client that prints what the daemon answered: the variant,
/// snake case, over its payload — `{"found":…}`, `{"error":…}` — and
/// a bare variant the string alone — `"not_found"`. The wire is the
/// tag byte and the payload, never this.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Frame {
    /// The compartment is gone: its role, its schema and every table
    /// in it. A dependency template's took the template's tools and its
    /// body with it. Tag `0`.
    Deleted,
    /// No compartment is the one named; nothing changed. Tag `1`.
    NotFound,
    /// The compartment goes with what it is of and by no delete of its
    /// own, or what would keep it is there still: a template that
    /// contains the image, an agent whose last registration declared
    /// the dependency template. Nothing changed. Tag `2`.
    NotDeletable,
    /// A container holds a connection through the compartment, or a
    /// dependency tool of its template runs. Nothing changed: wait for
    /// the container to stop, and ask again. Tag `3`.
    InUse,
    /// The account the request is served for holds no grant allowing
    /// it; nothing changed. Tag `4`.
    Forbidden,
    /// A failure. Tag `5`.
    ///
    /// Nothing changed. See
    /// [`shared::error::Error`](crate::shared::error::Error) for why it
    /// says so little.
    Error(Error),
}

/// Tag for [`Frame::Deleted`].
const DELETED: u8 = 0;

/// Tag for [`Frame::NotFound`].
const NOT_FOUND: u8 = 1;

/// Tag for [`Frame::NotDeletable`].
const NOT_DELETABLE: u8 = 2;

/// Tag for [`Frame::InUse`].
const IN_USE: u8 = 3;

/// Tag for [`Frame::Forbidden`].
const FORBIDDEN: u8 = 4;

/// Tag for [`Frame::Error`].
const ERROR: u8 = 5;

/// A tag, then the variant's own JSON, if it has any.
impl Encode for Frame {
    /// The ordinary JSON failure. The bare answers cannot fail.
    type Error = serde_json::Error;

    // Spelled out rather than `Self::Error`: this enum has a variant
    // called `Error`, so the associated type is ambiguous by that name.
    fn encode(&self, out: &mut Writer<'_>) -> Result<(), serde_json::Error> {
        match self {
            Frame::Deleted => {
                out.extend_from_slice(&[DELETED]);
                Ok(())
            }
            Frame::NotFound => {
                out.extend_from_slice(&[NOT_FOUND]);
                Ok(())
            }
            Frame::NotDeletable => {
                out.extend_from_slice(&[NOT_DELETABLE]);
                Ok(())
            }
            Frame::InUse => {
                out.extend_from_slice(&[IN_USE]);
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
    /// Each way to fail names which half failed.
    type Error = FrameError;

    // Spelled out for the same reason as `encode` above.
    fn decode(bytes: &[u8]) -> Result<Self, FrameError> {
        let (tag, rest) = bytes.split_first().ok_or(FrameError::Empty)?;
        match *tag {
            DELETED => Ok(Frame::Deleted),
            NOT_FOUND => Ok(Frame::NotFound),
            NOT_DELETABLE => Ok(Frame::NotDeletable),
            IN_USE => Ok(Frame::InUse),
            FORBIDDEN => Ok(Frame::Forbidden),
            ERROR => Error::decode(rest).map(Frame::Error).map_err(FrameError::Error),
            tag => Err(FrameError::UnknownTag(tag)),
        }
    }
}

/// A delete response frame that could not be read.
#[derive(Debug)]
pub enum FrameError {
    /// No bytes at all, so not even a tag.
    Empty,
    /// A tag that is none of this frame's six.
    UnknownTag(u8),
    /// The error did not parse.
    Error(serde_json::Error),
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameError::Empty => f.write_str("postgres compartments delete response frame is empty"),
            FrameError::UnknownTag(tag) => write!(f, "unknown postgres compartments delete response frame tag {tag}"),
            FrameError::Error(error) => write!(f, "postgres compartments delete error did not parse: {error}"),
        }
    }
}

impl std::error::Error for FrameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FrameError::Error(error) => Some(error),
            FrameError::Empty | FrameError::UnknownTag(_) => None,
        }
    }
}
