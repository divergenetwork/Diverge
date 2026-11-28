//! What a client's channel request frame carries for an agent
//! container begin.

use std::error::Error;
use std::fmt;

use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};
use crate::shared::containers::{daemon, enqueue, postgres};

/// What the server asks the proxy for once an agent container has
/// begun.
///
/// A payload leads with one byte saying which, and the rest is that
/// variant's own bytes.
///
/// | tag | asks for |
/// |-----|----------|
/// | `0` | [`Postgres`](Self::Postgres) |
/// | `1` | unused |
/// | `2` | [`Enqueue`](Self::Enqueue) |
/// | `3` | unused |
/// | `4` | unused |
/// | `5` | [`Daemon`](Self::Daemon) |
///
/// The first two tags are the same in both begin scopes — the second
/// unused in each — so a reader of
/// one is a reader of both; what follows is this family's own
/// exchange, and then the server's half of a daemon connection, which
/// a tool container's begin has under a tag of its own. All of
/// them reach INTO the container: the last hop of what a
/// caller opened on the provider. What
/// comes OUT of the agent — its conversation — is no channel: it
/// rides the begin's own main stream.
#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    /// The server's half of a database connection. Tag `0`.
    ///
    /// Opened once the server has taken the proxy's half, quoting the
    /// same connection; what comes back is everything the container's
    /// driver wrote. See
    /// [`postgres`](crate::shared::containers::postgres) for the pair.
    Postgres(postgres::request::Postgres),
    /// A batch of messages for the agent, in order. Tag `2`.
    ///
    /// The one way into it: a batch with no loop running starts one,
    /// on its messages, and a batch while one runs joins its queue.
    /// Answered once — by an
    /// [`enqueue::response::Frame`](crate::shared::containers::enqueue::response::Frame)
    /// naming the batch's fate, whenever that is known — and then the
    /// finish. What the agent says in reply is the begin's main stream.
    /// See [`enqueue`](crate::shared::containers::enqueue).
    Enqueue(enqueue::request::Request),
    /// A daemon connection the program opened, by the id the proxy
    /// announced: the server's half of it. Tag `5`.
    ///
    /// What comes back on it is everything the PROGRAM sends, as
    /// [`daemon::client::Frame`](crate::shared::containers::daemon::client::Frame)s;
    /// its finish is the program's socket ended. See
    /// [`daemon`](crate::shared::containers::daemon) for the pair.
    Daemon(daemon::request::Daemon),
}

/// Tag for [`Frame::Postgres`].
const POSTGRES: u8 = 0;

/// Tag for [`Frame::Enqueue`].
const ENQUEUE: u8 = 2;

/// Tag for [`Frame::Daemon`].
const DAEMON: u8 = 5;

impl Encode for Frame {
    /// The ordinary JSON failure, from whichever ask has one.
    type Error = serde_json::Error;

    fn encode(&self, out: &mut Writer<'_>) -> Result<(), Self::Error> {
        match self {
            Frame::Postgres(request) => {
                out.extend_from_slice(&[POSTGRES]);
                // Its error is `Infallible`, and an empty match on one
                // is how you say so: there is no value to handle.
                request.encode(out).map_err(|error| match error {})
            }
            Frame::Enqueue(request) => {
                out.extend_from_slice(&[ENQUEUE]);
                request.encode(out)
            }
            Frame::Daemon(request) => {
                out.extend_from_slice(&[DAEMON]);
                request.encode(out).map_err(|error| match error {})
            }
        }
    }
}

impl Decode<'_> for Frame {
    /// Several ways to fail, and the parses among them name which.
    type Error = FrameError;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        let (tag, rest) = bytes.split_first().ok_or(FrameError::Empty)?;
        match *tag {
            POSTGRES => postgres::request::Postgres::decode(rest)
                .map(Frame::Postgres)
                .map_err(FrameError::Postgres),
            ENQUEUE => enqueue::request::Request::decode(rest)
                .map(Frame::Enqueue)
                .map_err(FrameError::Enqueue),
            DAEMON => daemon::request::Daemon::decode(rest)
                .map(Frame::Daemon)
                .map_err(FrameError::Daemon),
            tag => Err(FrameError::UnknownTag(tag)),
        }
    }
}

/// An agents begin channel request that could not be read.
#[derive(Debug)]
pub enum FrameError {
    /// No bytes at all, so not even a tag.
    Empty,
    /// A tag that is none of this frame's.
    UnknownTag(u8),
    /// The connection id was not four bytes.
    Postgres(postgres::request::PostgresError),
    /// The enqueued message did not parse as JSON.
    Enqueue(serde_json::Error),
    /// The daemon connection's id was not four bytes.
    Daemon(daemon::request::DaemonError),
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameError::Empty => {
                f.write_str("agents begin channel request frame is empty")
            }
            FrameError::UnknownTag(tag) => {
                write!(f, "unknown agents begin channel request tag {tag}")
            }
            FrameError::Postgres(error) => write!(f, "{error}"),
            FrameError::Enqueue(error) => {
                write!(f, "enqueue request did not parse: {error}")
            }
            FrameError::Daemon(error) => write!(f, "{error}"),
        }
    }
}

impl Error for FrameError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FrameError::Enqueue(error) => Some(error),
            FrameError::Postgres(error) => Some(error),
            FrameError::Daemon(error) => Some(error),
            FrameError::Empty | FrameError::UnknownTag(_) => None,
        }
    }
}
