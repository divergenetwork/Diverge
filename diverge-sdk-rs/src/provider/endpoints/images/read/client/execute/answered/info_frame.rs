//! The document's answer, read whole.

use std::convert::Infallible;

use bytes::Bytes;

use crate::provider::endpoints::containers::client::Answered;
use crate::shared::containers::info::response;
use crate::wire::decode::Decode as _;

/// The one frame of an info channel, whole: the document, that the
/// image ships none, an error, or that the file is too large — see
/// [`response::Frame`]. It has no refusal: each of the four is an
/// answer, and a caller that carries the answer on, as a daemon does,
/// carries it as it came.
/// [`Info`](crate::provider::endpoints::containers::client::answered::Info)
/// reads the same frame for a caller that wants the document or its
/// absence and takes the rest as a refusal.
#[derive(Debug, Clone, Copy)]
pub struct InfoFrame;

impl Answered for InfoFrame {
    type Item = response::Frame;
    type Error = response::FrameError;
    type Refusal = Infallible;

    fn decode(payload: Bytes) -> Result<Result<response::Frame, Infallible>, Self::Error> {
        Ok(Ok(response::Frame::decode(&payload)?))
    }
}
