//! Bringing platforms of an image onto a provider: what a push and a
//! publish both do.

use std::sync::Arc;

use diverge_sdk::daemon::endpoints::images::push::server::response::{Added, Blob};
use diverge_sdk::provider::client::{Manifest, OciStore};
use diverge_sdk::provider::endpoints::images::push::client::execute::serve as supply;
use diverge_sdk::provider::endpoints::images::push::client::request::Frame as Ask;
use diverge_sdk::provider::endpoints::images::push::server::response::{Frame as Said, Pushed};
use diverge_sdk::shared::containers::oci;
use diverge_sdk::shared::containers::request::Image;
use diverge_sdk::shared::containers::response::Source as Came;
use diverge_sdk::shared::error::Error;
use diverge_sdk::wire::client::handle::Handle;
use diverge_sdk::wire::credit::Receiver;
use diverge_sdk::wire::decode::Decode as _;
use diverge_sdk::wire::encode::{Encode as _, Writer};
use diverge_sdk::wire::frame::server::ServerFrame;
use tokio::sync::mpsc;
use tokio::task::{JoinHandle, JoinSet};

use crate::containers::images;
use crate::daemon::Daemon;
use crate::serve::reply;

/// Something that happened while platforms were brought in, told as
/// it happened: what a push and a publish each send on as their own
/// frame.
pub enum Step {
    /// A blob the provider pulled from its caller, and verified.
    Blob(Blob),
    /// Platforms that are now held on the provider.
    Added(Added),
}

/// Where a [`Step`] is told.
pub type Steps = mpsc::UnboundedSender<Step>;

/// Why platforms were not brought in: the provider's refusals of a
/// push, and a failure.
pub enum Failure {
    /// A platform has no source, or a source did not supply all of it.
    Unavailable,
    /// The Store has no room.
    InsufficientCapacity,
    /// A source supplied the image and did not prove it holds it.
    ProofFailed,
    /// The image is not one the provider takes.
    Malformed,
    /// The provider has no Store.
    Unsupported,
    /// A platform named is not one the image has.
    PlatformNotFound,
    /// A failure, in the daemon's words or the provider's.
    Error(Error),
}

/// How one push to the provider ended.
pub enum Ended {
    /// The platforms named are in the Store: the provider's word.
    Pushed(Pushed),
    /// The daemon held every platform named there already: the digest.
    Exists(String),
    /// Nothing of it was added.
    Failed(Failure),
}

/// How [`bring`] ended.
pub enum Brought {
    /// Every platform is held, and at least one push moved something:
    /// the image's size as the provider counts it once the last is
    /// counted, and where the last to end took its platforms from.
    Pushed { bytes: u64, source: Came },
    /// Every platform was held already.
    Exists,
    /// A platform is not held: the first failure to end. What the
    /// other pushes added stays, and was told.
    Failed(Failure),
}

/// The platforms `platforms` of `image` brought into the daemon's
/// Store on the provider `handle` reaches, from wherever the daemon
/// finds them. The rules:
///
/// 1. **Searched as a read is.** Every provider connected, the one
///    pushed to among them, and every remote daemon recorded is asked
///    at once which platforms of the image it can supply, as
///    [`images::search`] asks. The provider pushed to answers with
///    what the daemon owns there already and what its registries
///    serve, so a platform held there needs no other source.
/// 2. **Nothing moves until every platform is found.** The search is
///    waited for to its end. A platform named that no source supplies
///    is [`Failure::Unavailable`]: every source found is let go, and
///    nothing is asked of the provider.
/// 3. **One push for each source.** Each source's platforms are one
///    push to the provider, naming those and no other, answered
///    through that source alone: see [`from_source`]. They run at
///    once, and none waits for another.
/// 4. **Told as it happens.** Each blob a push pulls is a
///    [`Step::Blob`]. Each push that ends with its platforms held is
///    one [`Step::Added`], told as it ends: with the bytes and where
///    the provider took them from, or, where the provider held them
///    all already, with no bytes and `store`.
/// 5. **The end.** Every push is waited for, whatever another ended
///    as: see [`Brought`].
pub async fn bring(daemon: &Arc<Daemon>, handle: &Handle, image: &Image, platforms: &[String], steps: &Steps) -> Brought {
    // Rule 1.
    let reach = images::Reach::Full { except: None };
    let mut search = images::search(daemon, image, reach, &[], images::Wanted::Every(platforms.to_vec()));
    let mut sources: Vec<(Arc<images::Source>, Vec<String>)> = Vec::new();
    while let Some(found) = search.next().await {
        match sources.iter_mut().find(|(source, _)| Arc::ptr_eq(source, &found.source)) {
            Some((_, platforms)) => platforms.extend(found.platforms),
            None => sources.push((found.source, found.platforms)),
        }
    }
    search.stop().await;
    // Rule 2.
    let found: usize = sources.iter().map(|(_, platforms)| platforms.len()).sum();
    if found != platforms.len() {
        for (source, _) in sources {
            images::let_go(source);
        }
        return Brought::Failed(Failure::Unavailable);
    }
    // Rule 3.
    let mut pushes = JoinSet::new();
    for (source, platforms) in sources {
        let ask = Ask {
            image: image.clone(),
            platforms: platforms.clone(),
        };
        let (handle, steps) = (handle.clone(), steps.clone());
        pushes.spawn(async move { (platforms, from_source(handle, ask, source, steps).await) });
    }
    // Rules 4 and 5.
    let mut pushed: Option<(u64, Came)> = None;
    let mut failed: Option<Failure> = None;
    while let Some(joined) = pushes.join_next().await {
        let (platforms, ended) = match joined {
            Ok(ended) => ended,
            Err(error) => {
                failed.get_or_insert(Failure::Error(reply::failure(&error)));
                continue;
            }
        };
        match ended {
            Ended::Pushed(said) => {
                let _ = steps.send(Step::Added(Added {
                    platforms,
                    bytes: Some(said.bytes),
                    source: said.source.clone(),
                }));
                let bytes = pushed.as_ref().map_or(said.bytes, |(bytes, _)| said.bytes.max(*bytes));
                pushed = Some((bytes, said.source));
            }
            Ended::Exists(_) => {
                let _ = steps.send(Step::Added(Added {
                    platforms,
                    bytes: None,
                    source: Came::Store,
                }));
            }
            Ended::Failed(failure) => {
                failed.get_or_insert(failure);
            }
        }
    }
    match (failed, pushed) {
        (Some(failure), _) => Brought::Failed(failure),
        (None, Some((bytes, source))) => Brought::Pushed { bytes, source },
        (None, None) => Brought::Exists,
    }
}

/// One push to the provider whose asks are answered from the one
/// `source` it is for: the provider's own push executor answers each
/// from that source's read scope, through [`Sourced`]. The source is
/// released when the push ends, however it ends.
async fn from_source(handle: Handle, ask: Ask, source: Arc<images::Source>, steps: Steps) -> Ended {
    let pulls = Arc::new(images::Pulls::new());
    pulls.keep(source).await;
    let ended = put(&handle, ask, &steps, |asks, remote, image| {
        let sourced = Arc::new(Sourced { pulls: Arc::clone(&pulls) });
        tokio::spawn(supply(asks, handle.clone(), remote, Arc::new(image), sourced))
    })
    .await;
    pulls.release().await;
    ended
}

/// One push to the provider, asked as the daemon itself, so that what
/// it brings in goes into the daemon's Store there. The rules:
///
/// 1. **The provider's asks are `answer`'s.** Once the push is open,
///    `answer` is handed the stream of channels the provider opens on
///    it, the push's scope and its image, and starts the task that
///    answers them: from a source, or by carrying each to a client.
/// 2. **The provider's answers.** Each blob pulled is a
///    [`Step::Blob`], told as it comes. Its `Pushed` and its `Exists`
///    end the push well; its six refusals are the [`Failure`] of the
///    same name; its error is the failure, in its words; a push that
///    would not open, an exchange that broke, or one that ended with
///    no result, is the failure, in the daemon's.
/// 3. **The answering ends with the push**: whatever answer is still
///    on its way then is let go.
pub async fn put<F>(handle: &Handle, ask: Ask, steps: &Steps, answer: F) -> Ended
where
    F: FnOnce(Receiver, u32, Image) -> JoinHandle<()>,
{
    let failed = |error: &dyn std::fmt::Display| Ended::Failed(Failure::Error(reply::failure(&error.to_string())));
    let mut payload = Vec::new();
    if let Err(error) = ask.encode(&mut Writer::new(&mut payload)) {
        return failed(&error);
    }
    let mut remote = match handle.send_request(&payload).await {
        Ok(remote) => remote,
        Err(error) => return failed(&error),
    };
    // Rule 1.
    let answering = answer(remote.request_receiver, remote.scope, ask.image);
    // Rule 2.
    let ended = loop {
        let Some(bytes) = remote.response_receiver.recv().await else {
            break failed(&"the connection to the provider ended with the push open");
        };
        let payload = match ServerFrame::decode(&bytes) {
            Ok(ServerFrame::Response { payload, .. }) => payload,
            Ok(ServerFrame::ResponseFinish { .. }) => break failed(&"the provider ended the push with no result"),
            Ok(_) => continue,
            Err(error) => break failed(&error),
        };
        match Said::decode(payload) {
            Ok(Said::Blob(blob)) => {
                let _ = steps.send(Step::Blob(Blob {
                    digest: blob.digest,
                    bytes: blob.bytes,
                }));
            }
            Ok(Said::Pushed(pushed)) => break Ended::Pushed(pushed),
            Ok(Said::Exists(exists)) => break Ended::Exists(exists.digest),
            Ok(Said::InsufficientCapacity) => break Ended::Failed(Failure::InsufficientCapacity),
            Ok(Said::Unavailable) => break Ended::Failed(Failure::Unavailable),
            Ok(Said::ProofFailed) => break Ended::Failed(Failure::ProofFailed),
            Ok(Said::Malformed) => break Ended::Failed(Failure::Malformed),
            Ok(Said::Unsupported) => break Ended::Failed(Failure::Unsupported),
            Ok(Said::PlatformNotFound) => break Ended::Failed(Failure::PlatformNotFound),
            Ok(Said::Error(error)) => break Ended::Failed(Failure::Error(error)),
            Err(error) => break failed(&error),
        }
    };
    // Rule 3.
    answering.abort();
    ended
}

/// A provider's asks on one push answered by the daemon, from the one
/// source the push is for, kept in `pulls`: read through it as a
/// run's asks are — see [`images`](crate::containers::images).
struct Sourced {
    /// The one scope held on the source.
    pulls: Arc<images::Pulls>,
}

impl OciStore for Sourced {
    async fn holds(&self, image: &Image) -> Option<String> {
        self.pulls.held().await.map(|_| images::name(&image.digest))
    }

    type Blob = images::Blob;

    async fn manifest(&self, _image: &Image, digest: &str) -> Option<Manifest> {
        images::manifest(&self.pulls, digest).await
    }

    async fn blob(&self, _image: &Image, digest: &str) -> Option<Self::Blob> {
        images::blob(&self.pulls, digest).await
    }

    async fn prove(&self, _image: &Image, request: &oci::prove::request::Request) -> Option<[u8; 32]> {
        images::prove(&self.pulls, request).await
    }
}
