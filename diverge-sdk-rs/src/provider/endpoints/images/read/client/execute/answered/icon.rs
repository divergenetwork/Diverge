//! An icon's answer, read.

use std::convert::Infallible;

use bytes::Bytes;

use crate::provider::endpoints::containers::client::Answered;
use crate::shared::containers::icon::response;
use crate::wire::decode::Decode as _;

/// The one frame of an icon's channel, whole: the file's bytes, that
/// the image ships no such icon, an error, or that the file is too
/// large — see [`response::Frame`]. It has no refusal: each of the
/// four is an answer, and a caller that carries the answer on, as a
/// daemon does, carries it as it came.
#[derive(Debug, Clone, Copy)]
pub struct Icon;

impl Answered for Icon {
    type Item = response::Frame;
    type Error = response::FrameError;
    type Refusal = Infallible;

    fn decode(payload: Bytes) -> Result<Result<response::Frame, Infallible>, Self::Error> {
        Ok(Ok(response::Frame::decode(&payload)?))
    }
}
