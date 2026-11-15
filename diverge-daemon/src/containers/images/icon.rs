//! An image's icons: the daemon's record of each, and the read that
//! fills the record.

use std::sync::Arc;

use diverge_sdk::shared::containers::icon::response::Frame;

use super::Pulls;
use crate::daemon::Daemon;
use crate::serve::reply;
use crate::store;
use crate::store::image_icons::Which;

/// One of an image's icons, for a client's icon channel on an
/// `images::read` scope the daemon holds open through `pulls`: the
/// answer as the channel carries it. It is read as
/// [`through`](super::through) reads the document. The rules:
///
/// 1. **The record first.** An icon the daemon has read before is on
///    its record by the image's digest and by which icon it is — a
///    digest is the image, so an icon found once is that icon for
///    good — and answers at once, with nobody asked.
/// 2. **Else the scope's own source**, already held, on that source's
///    channel for the same icon. With no source held any more, the
///    answer is an error.
/// 3. **The source's answer, as it came.** The bytes, that the image
///    ships none, an error, and that the file is too large are each
///    answered as the source answered them; a channel that broke is an
///    error in the daemon's words.
/// 4. **Only an icon is recorded.** The bytes are saved to the record;
///    every other answer is asked again the next time.
pub async fn icon(daemon: &Arc<Daemon>, pulls: &Pulls, digest: &str, which: Which) -> Frame {
    // Rule 1.
    match recorded(daemon, digest, which).await {
        Ok(Some(bytes)) => return Frame::Icon(bytes),
        Ok(None) => {}
        Err(error) => return Frame::Error(reply::failure(&error)),
    }
    // Rule 2.
    let Some(source) = pulls.held().await else {
        return Frame::Error(reply::failure(&"no source holds the image any more"));
    };
    let answered = match which {
        Which::Svg => source.scope.icon_svg().await,
        Which::Png512 => source.scope.icon_512().await,
        Which::Png64 => source.scope.icon_64().await,
    };
    // Rules 3 and 4.
    match answered {
        Ok(Frame::Icon(bytes)) => {
            if let Err(error) = record(daemon, digest, which, &bytes).await {
                return Frame::Error(reply::failure(&error));
            }
            Frame::Icon(bytes)
        }
        Ok(frame) => frame,
        Err(error) => Frame::Error(reply::failure(&error)),
    }
}

/// The record's icon for `digest`, if it has one.
async fn recorded(daemon: &Daemon, digest: &str, which: Which) -> Result<Option<Vec<u8>>, store::Error> {
    let mut conn = daemon.store.acquire().await?;
    store::image_icons::by_digest(&mut conn, digest, which).await
}

/// `bytes` put on the record for `digest`.
async fn record(daemon: &Daemon, digest: &str, which: Which, bytes: &[u8]) -> Result<(), store::Error> {
    let mut conn = daemon.store.acquire().await?;
    store::image_icons::save(&mut conn, digest, which, bytes).await
}
