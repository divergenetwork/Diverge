//! The publish, as it is told.

use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use futures_util::Stream;

use super::super::super::server::response;
use crate::shared::error::Error;
use crate::wire::decode::Decode as _;
use crate::wire::frame;

/// The publish, one [`response::Frame`] per item, in the order the
/// provider sends them: zero or more [`Blob`](response::Frame::Blob)s
/// uploaded, then the image [`Published`](response::Frame::Published),
/// then the end. A refusal and a provider's error are the last item,
/// as an [`ExecuteStreamError`], with nothing after it.
#[must_use = "a scope that is not polled grows a queue nobody reads"]
#[derive(Debug)]
pub struct ExecuteStream {
    /// The scope's responses, until the finish or an ending.
    receiver: Option<crate::wire::credit::Receiver>,
}

impl ExecuteStream {
    /// Over the scope's responses.
    pub(super) fn new(receiver: crate::wire::credit::Receiver) -> Self {
        ExecuteStream {
            receiver: Some(receiver),
        }
    }

    /// End the stream with `error` as its last item.
    fn end(&mut self, error: ExecuteStreamError) -> Poll<Option<Result<response::Frame, ExecuteStreamError>>> {
        self.receiver = None;
        Poll::Ready(Some(Err(error)))
    }
}

impl Stream for ExecuteStream {
    type Item = Result<response::Frame, ExecuteStreamError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Some(receiver) = &mut self.receiver else {
            return Poll::Ready(None);
        };
        let Some(bytes) = ready!(receiver.poll_recv(cx)) else {
            return self.end(ExecuteStreamError::Closed);
        };
        let envelope = match frame::server::ServerFrame::decode(&bytes) {
            Ok(envelope) => envelope,
            Err(error) => return self.end(ExecuteStreamError::Frame(error)),
        };
        let payload = match envelope {
            frame::server::ServerFrame::Response { payload, .. } => payload,
            frame::server::ServerFrame::ResponseFinish { .. } => {
                self.receiver = None;
                return Poll::Ready(None);
            }
            _ => return self.end(ExecuteStreamError::Misrouted),
        };
        match response::Frame::decode(payload) {
            Ok(response::Frame::NotFound) => self.end(ExecuteStreamError::NotFound),
            Ok(response::Frame::Incomplete) => self.end(ExecuteStreamError::Incomplete),
            Ok(response::Frame::RegistryUnlisted) => self.end(ExecuteStreamError::RegistryUnlisted),
            Ok(response::Frame::Malformed) => self.end(ExecuteStreamError::Malformed),
            Ok(response::Frame::Unauthorized) => self.end(ExecuteStreamError::Unauthorized),
            Ok(response::Frame::Denied) => self.end(ExecuteStreamError::Denied),
            Ok(response::Frame::Unsupported) => self.end(ExecuteStreamError::Unsupported),
            Ok(response::Frame::Error(error)) => self.end(ExecuteStreamError::Refused(error)),
            Ok(frame) => Poll::Ready(Some(Ok(frame))),
            Err(error) => self.end(ExecuteStreamError::Response(error)),
        }
    }
}

/// Why the stream stopped short of the image published.
#[derive(Debug)]
pub enum ExecuteStreamError {
    /// The connection went away with the scope still open.
    Closed,
    /// A frame that would not decode.
    Frame(frame::FrameError),
    /// A frame that cannot be on a main stream.
    Misrouted,
    /// A response that is not this scope's.
    Response(response::FrameDecodeError),
    /// The caller owns no image by the digest. The protocol's defined
    /// refusal, not a failure, as each of the six after it is.
    NotFound,
    /// The image does not hold every platform its manifest names.
    Incomplete,
    /// The registry is none of the provider's.
    RegistryUnlisted,
    /// The repository path or a tag is not one.
    Malformed,
    /// The registry did not take the credential.
    Unauthorized,
    /// The credential may not write to the repository.
    Denied,
    /// The provider has no Store.
    Unsupported,
    /// The provider ended the publish with an error, in its own
    /// words. See [`shared::error::Error`](crate::shared::error::Error)
    /// for why it says so little.
    Refused(Error),
}

impl fmt::Display for ExecuteStreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecuteStreamError::Closed => f.write_str("the connection ended with the image publish open"),
            ExecuteStreamError::Frame(error) => write!(f, "frame did not decode: {error}"),
            ExecuteStreamError::Misrouted => f.write_str("a frame that cannot be on a main stream arrived"),
            ExecuteStreamError::Response(error) => write!(f, "{error}"),
            ExecuteStreamError::NotFound => f.write_str("the caller owns no image by that digest"),
            ExecuteStreamError::Incomplete => f.write_str("the image does not hold every platform its manifest names"),
            ExecuteStreamError::RegistryUnlisted => f.write_str("the registry is none of the provider's"),
            ExecuteStreamError::Malformed => f.write_str("the repository path or a tag is not one"),
            ExecuteStreamError::Unauthorized => f.write_str("the registry did not take the credential"),
            ExecuteStreamError::Denied => f.write_str("the credential may not write to the repository"),
            ExecuteStreamError::Unsupported => f.write_str("the provider has no Store"),
            ExecuteStreamError::Refused(_) => f.write_str("the provider ended the image publish with an error"),
        }
    }
}

impl std::error::Error for ExecuteStreamError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ExecuteStreamError::Frame(error) => Some(error),
            ExecuteStreamError::Response(error) => Some(error),
            ExecuteStreamError::Closed
            | ExecuteStreamError::Misrouted
            | ExecuteStreamError::NotFound
            | ExecuteStreamError::Incomplete
            | ExecuteStreamError::RegistryUnlisted
            | ExecuteStreamError::Malformed
            | ExecuteStreamError::Unauthorized
            | ExecuteStreamError::Denied
            | ExecuteStreamError::Unsupported
            | ExecuteStreamError::Refused(_) => None,
        }
    }
}
