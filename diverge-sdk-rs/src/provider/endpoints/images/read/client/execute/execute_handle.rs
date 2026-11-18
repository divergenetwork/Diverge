//! A read scope held open, and everything a client can ask on it.

use std::fmt;

use tokio::sync::Mutex;

use super::super::channel_request;
use super::answered::{Icon, InfoFrame, OciBlob, OciManifest, OciProve};
use crate::shared::containers::{icon, info};
use crate::provider::client::Manifest;
use crate::provider::endpoints::containers::client::answered::Info;
use crate::provider::endpoints::containers::client::{ChannelStream, OpenError, UnaryError, unary};
use crate::shared::containers::info::Document;
use crate::shared::containers::oci;
use crate::wire::client::handle::Handle;
use crate::wire::encode::{Encode, Writer};
use crate::wire::frame;

/// A blob's pieces, as the provider sends them, for as long as the
/// channel lives; the finish ends it, and zero pieces before the
/// finish is a digest the scope's image does not reach.
pub type BlobStream = ChannelStream<OciBlob>;

/// The read scope of one image, answered held, and held until it is
/// cancelled.
///
/// Every channel a client may open on a read scope is a method here:
/// the three asks, the document, its three icons, and the cancel. Each opens its own channel, so
/// several may be in flight at once. The readers are the run's: the
/// same shared frames answer the same three asks there.
///
/// # Dropping it does nothing
///
/// No destructor sends a frame. Ending a read scope is
/// [`cancel`](Self::cancel), or the connection going; the provider
/// finishes the scope on either.
pub struct ExecuteHandle {
    handle: Handle,
    scope: u32,
    platforms: Vec<String>,
    main: Mutex<crate::wire::credit::Receiver>,
}

/// What a read scope says on its main stream after its first held:
/// what [`more`](ExecuteHandle::more) reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum More {
    /// More platforms found: a daemon's held, after its first.
    Held(Vec<String>),
    /// A daemon looks no further: nothing more is found on this scope,
    /// which stays open to be asked on.
    Settled,
    /// The scope ended.
    End(End),
}

/// The tag a held leads with on a read scope's main stream, a
/// provider's and a daemon's alike.
const HELD: u8 = 0;

/// The tag a daemon's settled is, on its read scope's main stream.
const SETTLED: u8 = 4;

/// One channel request of this scope, as its bytes.
fn payload(frame: &channel_request::Frame) -> Result<Vec<u8>, serde_json::Error> {
    let mut bytes = Vec::new();
    frame.encode(&mut Writer::new(&mut bytes))?;
    Ok(bytes)
}

impl ExecuteHandle {
    pub(crate) fn new(handle: Handle, scope: u32, main: crate::wire::credit::Receiver, platforms: Vec<String>) -> Self {
        ExecuteHandle {
            handle,
            scope,
            platforms,
            main: Mutex::new(main),
        }
    }

    /// The platforms the first held carried: all a provider can
    /// supply, or the first a daemon found.
    pub fn platforms(&self) -> &[String] {
        &self.platforms
    }

    /// What the scope says next on its main stream. The rules:
    ///
    /// 1. **A daemon's scope goes on.** After its first held a daemon
    ///    sends a held for each source it finds that adds platforms,
    ///    carrying those and no other, then exactly one settled: each
    ///    is one [`More`].
    /// 2. **A provider's does not.** It says nothing more until the
    ///    finish, which is the [`End`].
    /// 3. **Read in order, by one reader.** This and
    ///    [`wait`](Self::wait) read the one stream: a caller reads
    ///    every `more` to the settled before it waits for the end.
    ///    Anything else that arrives is passed over.
    pub async fn more(&self) -> More {
        let mut main = self.main.lock().await;
        while let Some(bytes) = main.recv().await {
            match frame::server::ServerFrame::decode(&bytes) {
                Ok(frame::server::ServerFrame::ResponseFinish { .. }) => return More::End(End::Finished),
                Ok(frame::server::ServerFrame::Response { payload, .. }) => match payload.split_first() {
                    Some((&HELD, rest)) => {
                        if let Ok(platforms) = serde_json::from_slice(rest) {
                            return More::Held(platforms);
                        }
                    }
                    Some((&SETTLED, _)) => return More::Settled,
                    _ => {}
                },
                Ok(_) => {}
                Err(_) => return More::End(End::Closed),
            }
        }
        More::End(End::Closed)
    }

    /// Ask a daemon's scope to look no further: see
    /// [`Enough`](channel_request::Frame::Enough). Nothing but the
    /// finish answers on the channel this opens.
    pub async fn enough(&self) -> Result<(), OpenError> {
        let payload = payload(&channel_request::Frame::Enough).map_err(OpenError::Request)?;
        self.handle.send_channel_request(self.scope, &payload).await.map(|_| ()).map_err(OpenError::Send)
    }

    /// The scope's number, as this end minted it.
    pub fn scope(&self) -> u32 {
        self.scope
    }

    /// The manifest under `digest`, of the scope's image: `None` for
    /// the empty finish, a digest the image does not reach.
    pub async fn manifest(&self, digest: &str) -> Result<Option<Manifest>, UnaryError<OciManifest>> {
        let request = oci::manifest::request::Request {
            digest: digest.to_string(),
        };
        let payload = payload(&channel_request::Frame::OciManifest(request)).map_err(UnaryError::Request)?;
        match unary::<OciManifest>(&self.handle, self.scope, &payload).await {
            Ok(manifest) => Ok(Some(manifest)),
            Err(UnaryError::Unanswered) => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// The blob under `digest`, piece by piece, for as long as the
    /// channel lives: every piece appends, the finish ends it, and a
    /// stream that ends with no piece is a digest the image does not
    /// reach.
    pub async fn blob(&self, digest: &str) -> Result<BlobStream, OpenError> {
        let request = oci::blob::request::Request {
            digest: digest.to_string(),
        };
        let payload = payload(&channel_request::Frame::OciBlob(request)).map_err(OpenError::Request)?;
        let channel = self.handle.send_channel_request(self.scope, &payload).await.map_err(OpenError::Send)?;
        Ok(ChannelStream::new(channel.response_receiver))
    }

    /// The proof over the scope's image, under `request`'s platform and
    /// nonce: `None` for the empty finish, a platform that breaks the
    /// rule of [`oci::prove`](crate::shared::containers::oci::prove).
    pub async fn prove(&self, request: &oci::prove::request::Request) -> Result<Option<[u8; 32]>, UnaryError<OciProve>> {
        let payload = payload(&channel_request::Frame::OciProve(request.clone())).map_err(UnaryError::Request)?;
        match unary::<OciProve>(&self.handle, self.scope, &payload).await {
            Ok(proof) => Ok(Some(proof)),
            Err(UnaryError::Unanswered) => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// The scope's image's `/.diverge/DIVERGE.md`, parsed: `None` for an image
    /// that ships none. The provider reads the file out of the image
    /// without running it; a file it cannot read, or that is no
    /// document, is the refusal.
    pub async fn info(&self) -> Result<Option<Document>, UnaryError<Info>> {
        let payload = payload(&channel_request::Frame::Info).map_err(UnaryError::Request)?;
        unary::<Info>(&self.handle, self.scope, &payload).await
    }

    /// The answer on the scope's info channel, whole: the document,
    /// that the image ships none, an error, or that the file is larger
    /// than [`MAX_BYTES`](info::MAX_BYTES). What [`info`](Self::info)
    /// reads, for a caller that carries the answer on as it came.
    pub async fn info_frame(&self) -> Result<info::response::Frame, UnaryError<InfoFrame>> {
        let payload = payload(&channel_request::Frame::Info).map_err(UnaryError::Request)?;
        unary::<InfoFrame>(&self.handle, self.scope, &payload).await
    }

    /// The scope's image's SVG icon, its `/.diverge/icon.svg`: the
    /// answer whole — the file's bytes, that the
    /// image ships none, an error, or that the file is larger than
    /// [`MAX_BYTES`](info::MAX_BYTES). The provider reads the file out
    /// of the image without running it.
    pub async fn icon_svg(&self) -> Result<icon::response::Frame, UnaryError<Icon>> {
        self.icon(channel_request::Frame::IconSvg).await
    }

    /// The scope's image's 512 by 512 PNG icon, its
    /// `/.diverge/icon-512.png`: as [`icon_svg`](Self::icon_svg).
    pub async fn icon_512(&self) -> Result<icon::response::Frame, UnaryError<Icon>> {
        self.icon(channel_request::Frame::Icon512).await
    }

    /// The scope's image's 64 by 64 PNG icon, its
    /// `/.diverge/icon-64.png`: as [`icon_svg`](Self::icon_svg).
    pub async fn icon_64(&self) -> Result<icon::response::Frame, UnaryError<Icon>> {
        self.icon(channel_request::Frame::Icon64).await
    }

    /// One icon's channel, opened and read.
    async fn icon(&self, ask: channel_request::Frame) -> Result<icon::response::Frame, UnaryError<Icon>> {
        let payload = payload(&ask).map_err(UnaryError::Request)?;
        unary::<Icon>(&self.handle, self.scope, &payload).await
    }

    /// End the scope. Nothing answers on the channel this opens; what
    /// answers is the scope's own finish, on [`wait`](Self::wait).
    pub async fn cancel(&self) -> Result<(), OpenError> {
        let payload = payload(&channel_request::Frame::Cancel).map_err(OpenError::Request)?;
        self.handle.send_channel_request(self.scope, &payload).await.map(|_| ()).map_err(OpenError::Send)
    }

    /// The scope's end: the finish, after the cancel, the provider
    /// ending the scope on its own, or the connection going first. The
    /// held was read by [`execute`](super::execute); anything else that
    /// arrives before the finish — a daemon's later helds and its
    /// settled, where nobody read them with [`more`](Self::more) — is
    /// passed over. Resolves once and answers the same way
    /// again after.
    pub async fn wait(&self) -> End {
        let mut main = self.main.lock().await;
        while let Some(bytes) = main.recv().await {
            match frame::server::ServerFrame::decode(&bytes) {
                Ok(frame::server::ServerFrame::ResponseFinish { .. }) => return End::Finished,
                Ok(_) => {}
                Err(_) => return End::Closed,
            }
        }
        End::Closed
    }
}

/// How a read scope answered held ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum End {
    /// The finish came: the cancel was answered, or the provider ended
    /// the scope for a reason it does not say.
    Finished,
    /// The connection ended before the finish.
    Closed,
}

impl fmt::Debug for ExecuteHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecuteHandle").field("scope", &self.scope).finish_non_exhaustive()
    }
}
