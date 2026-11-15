//! An icon's answer, read.

use bytes::Bytes;

use crate::provider::endpoints::containers::client::Answered;
use crate::shared::containers::icon::response;
use crate::shared::containers::info::MAX_BYTES;
use crate::shared::error::Error;
use crate::wire::decode::Decode as _;

/// The one frame of an icon's channel: the file's bytes — `None` when
/// the image ships no such icon, which is a fact about the image and
/// not a refusal — or the provider's refusal: the image's document
/// could not be read, or the file is larger than [`MAX_BYTES`], which
/// is refused in this side's own words, of kind `icon`.
#[derive(Debug, Clone, Copy)]
pub struct Icon;

impl Answered for Icon {
    type Item = Option<Vec<u8>>;
    type Error = response::FrameError;
    type Refusal = Error;

    fn decode(payload: Bytes) -> Result<Result<Option<Vec<u8>>, Error>, Self::Error> {
        Ok(match response::Frame::decode(&payload)? {
            response::Frame::Icon(bytes) => Ok(Some(bytes)),
            response::Frame::Missing => Ok(None),
            response::Frame::Error(error) => Err(error),
            response::Frame::TooLarge => Err(Error(serde_json::json!({
                "kind": "icon",
                "error": format!("the icon is larger than {MAX_BYTES} bytes"),
            }))),
        })
    }
}
