//! What a client's channel request frame carries for an agent container
//! run.

use std::error::Error;
use std::fmt;

use crate::wire::decode::Decode;
use crate::wire::encode::{Encode, Writer};
use crate::shared::containers::exec;
use crate::shared::containers::frontend;
use crate::shared::containers::terminal;
use crate::shared::containers::enqueue;
use crate::shared::containers::{daemon, fuse_mount, postgres, read, transfer, write_path};

/// What a caller asks a provider for while an agent container runs.
///
/// A payload leads with one byte saying which, and the rest is that
/// variant's own bytes.
///
/// | tag | asks for |
/// |-----|----------|
/// | `0` | [`Cancel`](Self::Cancel) |
/// | `1` | [`Filetree`](Self::Filetree) |
/// | `2` | [`Read`](Self::Read) |
/// | `3` | [`Write`](Self::Write) |
/// | `4` | [`Transfer`](Self::Transfer) |
/// | `5` | [`Postgres`](Self::Postgres) |
/// | `6` | unused |
/// | `7` | [`Enqueue`](Self::Enqueue) |
/// | `8` | unused |
/// | `9` | [`FuseMount`](Self::FuseMount) |
/// | `10` | unused |
/// | `11` | [`Exec`](Self::Exec) |
/// | `12` | [`TerminalOpen`](Self::TerminalOpen) |
/// | `13` | [`TerminalAttach`](Self::TerminalAttach) |
/// | `14` | [`TerminalStdin`](Self::TerminalStdin) |
/// | `15` | [`TerminalResize`](Self::TerminalResize) |
/// | `16` | [`TerminalClose`](Self::TerminalClose) |
/// | `17` | [`Frontend`](Self::Frontend) |
/// | `18` | [`Daemon`](Self::Daemon) |
///
/// The first seven tags are the same in every container scope — the
/// seventh unused in each, where an image's document was once asked
/// for and is now read on the image's own scope — in the same
/// order, so a reader of one is a reader of all; what follows is this
/// family's own exchange,
/// and then the mount, which every scope has again, under a tag of
/// its own, and last this end's half of a daemon connection, which a
/// tool container's scope has under a tag of its own. Tag `10` is unused: it once carried a feed of values to
/// the program, and it is never given to another request. All of
/// them but the first reach INTO the
/// container, which is the thing a caller cannot dial: it runs on the
/// provider. That is the whole reason these channels open outward
/// from the client rather than the other way. What comes OUT of the
/// agent — its conversation — is no channel: it rides the scope's own
/// main stream.
#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    /// Cancel the run: stop the container. Tag `0`.
    ///
    /// # It has no answer, and does not need one
    ///
    /// Nothing comes back on this channel. What comes back is the end
    /// of the SCOPE — a
    /// [`ResponseFinish`](crate::wire::frame::server::ServerFrame::ResponseFinish),
    /// which already means nothing bearing this scope follows on any
    /// channel. Finishing this one first would be a smaller way of
    /// saying the same thing, moments earlier.
    ///
    /// # What it adds over closing the connection
    ///
    /// The scope IS the container's life, so dropping the connection
    /// stops it too. The difference is that a provider cannot tell a
    /// deliberate exit from a network that stopped answering, and has
    /// to wait to find out. This is unambiguous and immediate: a
    /// caller that says so is not gone, it is finished.
    ///
    Cancel,
    /// The container's filesystem, watched. Tag `1`.
    ///
    /// Carries nothing — the variant is bare — and the provider answers
    /// with a snapshot and then every change, for as long as the
    /// channel lives. See
    /// [`filetree`](crate::shared::containers::filetree).
    Filetree,
    /// One file, read out of the container. Tag `2`.
    ///
    /// See [`read`](crate::shared::containers::read) for why this is
    /// one file and never a directory.
    Read(read::request::Request),
    /// One file, written into the container. Tag `3`.
    ///
    /// Carries no content. The provider answers by opening a channel
    /// of its own asking for it — see
    /// [`write_path`](crate::shared::containers::write_path) for why
    /// it travels that direction, and
    /// [`write_bytes`](crate::shared::containers::write_bytes) for
    /// what comes back.
    Write(write_path::request::Request),
    /// One file, copied into another container. Tag `4`.
    ///
    /// The other container by its id, and the caller must be running
    /// it. The provider reads the file out of this
    /// container and writes it into that one on its own connections
    /// to the two proxies, and nothing of the file comes back here —
    /// one answer does. See
    /// [`transfer`](crate::shared::containers::transfer) for the rule.
    Transfer(transfer::request::Request),
    /// The caller's half of a database connection. Tag `5`.
    ///
    /// Opened once the caller has taken the provider's half, quoting
    /// the same connection; what comes back is everything the
    /// container wrote. See
    /// [`postgres`](crate::shared::containers::postgres) for the pair.
    Postgres(postgres::request::Postgres),
    /// A batch of messages for the agent, in order. Tag `7`.
    ///
    /// The one way into it: a batch with no loop running starts one,
    /// on its messages, and a batch while one runs joins its queue.
    /// Answered once — by an
    /// [`enqueue::response::Frame`](crate::shared::containers::enqueue::response::Frame)
    /// naming the batch's fate, whenever that is known — and then the
    /// finish. What the agent says in reply is the scope's main stream.
    /// See [`enqueue`](crate::shared::containers::enqueue).
    Enqueue(enqueue::request::Request),
    /// A FUSE mount made while the container runs. Tag `9`.
    ///
    /// The caller serves one more file or directory into the
    /// container at a path, under an id of its own, as the mounts of
    /// the run's request are served. Answered once — by a
    /// [`fuse_mount::response::Frame`](crate::shared::containers::fuse_mount::response::Frame)
    /// saying the mount is made, or why not — and then the finish;
    /// the answer precedes every ask of the id. See
    /// [`fuse_mount`](crate::shared::containers::fuse_mount).
    FuseMount(fuse_mount::request::Request),
    /// Run one command in the container, its output streamed: the
    /// provider opens an exec scope on the proxy and relays the stream,
    /// as [`exec`](crate::shared::containers::exec) states it. The
    /// caller abandoning the channel kills the command.
    Exec(exec::request::Request),
    /// Start a terminal the provider holds, answered by its id (see
    /// [`session`](crate::shared::containers::terminal::session)).
    TerminalOpen(terminal::open::Open),
    /// Stream a held terminal's output: its scrollback, then live.
    TerminalAttach(terminal::session::Id),
    /// Write bytes to a held terminal's stdin.
    TerminalStdin {
        /// The terminal.
        id: u32,
        /// What is written.
        bytes: Vec<u8>,
    },
    /// Set a held terminal's size.
    TerminalResize(terminal::session::Resize),
    /// Kill a held terminal's shell.
    TerminalClose(terminal::session::Id),
    /// One HTTP request to the program's frontend, its body asked for
    /// by its write id as a write's content is (see
    /// [`frontend`](crate::shared::containers::frontend)).
    Frontend(frontend::request::Request),
    /// A daemon connection the container's program opened, by the id
    /// the provider announced: this end's half of it. Tag `18`.
    ///
    /// What comes back on it is everything the PROGRAM sends, as
    /// [`daemon::client::Frame`](crate::shared::containers::daemon::client::Frame)s;
    /// its finish is the program's socket ended. See
    /// [`daemon`](crate::shared::containers::daemon) for the pair.
    Daemon(daemon::request::Daemon),
}

/// Tag for [`Frame::Cancel`].
const CANCEL: u8 = 0;

/// Tag for [`Frame::Filetree`].
const FILETREE: u8 = 1;

/// Tag for [`Frame::Read`].
const READ: u8 = 2;

/// Tag for [`Frame::Write`].
const WRITE: u8 = 3;

/// Tag for [`Frame::Transfer`].
const TRANSFER: u8 = 4;

/// Tag for [`Frame::Postgres`].
const POSTGRES: u8 = 5;

/// Tag for [`Frame::Enqueue`].
const ENQUEUE: u8 = 7;

/// Tag for [`Frame::FuseMount`].
const FUSE_MOUNT: u8 = 9;

/// Tag for [`Frame::Exec`].
const EXEC: u8 = 11;

/// Tag for [`Frame::TerminalOpen`].
const TERMINAL_OPEN: u8 = 12;

/// Tag for [`Frame::TerminalAttach`].
const TERMINAL_ATTACH: u8 = 13;

/// Tag for [`Frame::TerminalStdin`].
const TERMINAL_STDIN: u8 = 14;

/// Tag for [`Frame::TerminalResize`].
const TERMINAL_RESIZE: u8 = 15;

/// Tag for [`Frame::TerminalClose`].
const TERMINAL_CLOSE: u8 = 16;

/// Tag for [`Frame::Frontend`].
const FRONTEND: u8 = 17;

/// Tag for [`Frame::Daemon`].
const DAEMON: u8 = 18;

impl Encode for Frame {
    /// The ordinary JSON failure, from whichever half has one.
    type Error = serde_json::Error;

    fn encode(&self, out: &mut Writer<'_>) -> Result<(), Self::Error> {
        match self {
            Frame::Cancel => {
                out.extend_from_slice(&[CANCEL]);
                Ok(())
            }
            Frame::Filetree => {
                out.extend_from_slice(&[FILETREE]);
                Ok(())
            }
            Frame::Read(request) => {
                out.extend_from_slice(&[READ]);
                request.encode(out)
            }
            Frame::Write(request) => {
                out.extend_from_slice(&[WRITE]);
                request.encode(out)
            }
            Frame::Transfer(request) => {
                out.extend_from_slice(&[TRANSFER]);
                request.encode(out)
            }
            Frame::Postgres(request) => {
                out.extend_from_slice(&[POSTGRES]);
                request.encode(out).map_err(|error| match error {})
            }
            Frame::Enqueue(request) => {
                out.extend_from_slice(&[ENQUEUE]);
                request.encode(out)
            }
            Frame::FuseMount(request) => {
                out.extend_from_slice(&[FUSE_MOUNT]);
                request.encode(out)
            }
            Frame::Exec(request) => {
                out.extend_from_slice(&[EXEC]);
                request.encode(out)
            }
            Frame::TerminalOpen(size) => {
                out.extend_from_slice(&[TERMINAL_OPEN]);
                size.encode(out)
            }
            Frame::TerminalAttach(id) => {
                out.extend_from_slice(&[TERMINAL_ATTACH]);
                serde_json::to_writer(out, id)
            }
            Frame::TerminalStdin { id, bytes } => {
                out.extend_from_slice(&[TERMINAL_STDIN]);
                terminal::session::Stdin { id: *id, bytes }.encode(out).unwrap_or_else(|never| match never {});
                Ok(())
            }
            Frame::TerminalResize(resize) => {
                out.extend_from_slice(&[TERMINAL_RESIZE]);
                serde_json::to_writer(out, resize)
            }
            Frame::TerminalClose(id) => {
                out.extend_from_slice(&[TERMINAL_CLOSE]);
                serde_json::to_writer(out, id)
            }
            Frame::Frontend(request) => {
                out.extend_from_slice(&[FRONTEND]);
                serde_json::to_writer(out, request)
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
            CANCEL => Ok(Frame::Cancel),
            FILETREE => Ok(Frame::Filetree),
            READ => read::request::Request::decode(rest)
                .map(Frame::Read)
                .map_err(FrameError::Read),
            WRITE => write_path::request::Request::decode(rest)
                .map(Frame::Write)
                .map_err(FrameError::Write),
            TRANSFER => transfer::request::Request::decode(rest)
                .map(Frame::Transfer)
                .map_err(FrameError::Transfer),
            POSTGRES => postgres::request::Postgres::decode(rest)
                .map(Frame::Postgres)
                .map_err(FrameError::Postgres),
            ENQUEUE => enqueue::request::Request::decode(rest)
                .map(Frame::Enqueue)
                .map_err(FrameError::Enqueue),
            FUSE_MOUNT => fuse_mount::request::Request::decode(rest)
                .map(Frame::FuseMount)
                .map_err(FrameError::FuseMount),
            EXEC => exec::request::Request::decode(rest)
                .map(Frame::Exec)
                .map_err(FrameError::Exec),
            TERMINAL_OPEN => terminal::open::Open::decode(rest)
                .map(Frame::TerminalOpen)
                .map_err(FrameError::Terminal),
            TERMINAL_ATTACH => serde_json::from_slice(rest)
                .map(Frame::TerminalAttach)
                .map_err(FrameError::Terminal),
            TERMINAL_STDIN => terminal::session::Stdin::decode(rest)
                .map(|stdin| Frame::TerminalStdin { id: stdin.id, bytes: stdin.bytes.to_vec() })
                .map_err(|_| FrameError::TerminalStdin),
            TERMINAL_RESIZE => serde_json::from_slice(rest)
                .map(Frame::TerminalResize)
                .map_err(FrameError::Terminal),
            TERMINAL_CLOSE => serde_json::from_slice(rest)
                .map(Frame::TerminalClose)
                .map_err(FrameError::Terminal),
            FRONTEND => serde_json::from_slice(rest)
                .map(Frame::Frontend)
                .map_err(FrameError::Frontend),
            DAEMON => daemon::request::Daemon::decode(rest)
                .map(Frame::Daemon)
                .map_err(FrameError::Daemon),
            tag => Err(FrameError::UnknownTag(tag)),
        }
    }
}

/// An agent container run channel request that could not be read.
#[derive(Debug)]
pub enum FrameError {
    /// No bytes at all, so not even a tag.
    Empty,
    /// A tag that is none of this frame's.
    UnknownTag(u8),
    /// The read request did not parse.
    Read(serde_json::Error),
    /// The write request did not parse.
    Write(serde_json::Error),
    /// The transfer request did not parse.
    Transfer(serde_json::Error),
    /// The connection id was not four bytes.
    Postgres(postgres::request::PostgresError),
    /// The enqueued message did not parse as JSON.
    Enqueue(serde_json::Error),
    /// The mount request did not parse.
    FuseMount(serde_json::Error),
    /// The exec request did not parse.
    Exec(serde_json::Error),
    /// A terminal request did not parse.
    Terminal(serde_json::Error),
    /// A terminal's stdin held no id.
    TerminalStdin,
    /// The frontend request did not parse.
    Frontend(serde_json::Error),
    /// The daemon connection's id was not four bytes.
    Daemon(daemon::request::DaemonError),
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameError::Empty => {
                f.write_str("agents run channel request frame is empty")
            }
            FrameError::UnknownTag(tag) => {
                write!(f, "unknown agents run channel request tag {tag}")
            }
            FrameError::Read(error) => {
                write!(f, "read request did not parse: {error}")
            }
            FrameError::Write(error) => {
                write!(f, "write request did not parse: {error}")
            }
            FrameError::Transfer(error) => {
                write!(f, "transfer request did not parse: {error}")
            }
            FrameError::Postgres(error) => write!(f, "{error}"),
            FrameError::Enqueue(error) => {
                write!(f, "enqueue request did not parse: {error}")
            }
            FrameError::FuseMount(error) => {
                write!(f, "fuse mount request did not parse: {error}")
            }
            FrameError::Exec(error) => {
                write!(f, "exec request did not parse: {error}")
            }
            FrameError::Terminal(error) => {
                write!(f, "terminal request did not parse: {error}")
            }
            FrameError::TerminalStdin => f.write_str("terminal stdin held no id"),
            FrameError::Frontend(error) => {
                write!(f, "frontend request did not parse: {error}")
            }
            FrameError::Daemon(error) => write!(f, "{error}"),
        }
    }
}

impl Error for FrameError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FrameError::Read(error)
            | FrameError::Write(error)
            | FrameError::Transfer(error)
            | FrameError::Enqueue(error)
            | FrameError::FuseMount(error)
            | FrameError::Exec(error)
            | FrameError::Terminal(error)
            | FrameError::Frontend(error) => Some(error),
            FrameError::Postgres(error) => Some(error),
            FrameError::Daemon(error) => Some(error),
            FrameError::Empty | FrameError::UnknownTag(_) | FrameError::TerminalStdin => None,
        }
    }
}
