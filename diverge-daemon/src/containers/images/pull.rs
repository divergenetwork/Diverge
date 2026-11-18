//! Pulling a manifest, a blob or the proof of an image through the
//! held sources' `images::read` scopes.

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use bytes::Bytes;
use diverge_sdk::provider::client::Manifest;
use diverge_sdk::shared::containers::oci;
use futures_util::future::ready;
use futures_util::stream::FuturesUnordered;
use futures_util::{Stream, StreamExt as _, stream};
use tokio::sync::Mutex;

use super::{Source, let_go};

/// A blob as it is relayed: the pieces of one source's answer, in
/// order, ending at its finish or short where the source went.
pub type Blob = Pin<Box<dyn Stream<Item = Bytes> + Send>>;

/// The `images::read` scopes held for an image: the sources a
/// [`search`](super::search) found, and the way to each.
///
/// The rules:
///
/// 1. **The sources found.** A scope is kept here only once a search
///    found it supplying a platform nobody before it did. One source
///    is the common case: a run's, a document's. Several are an image
///    whose platforms are spread over them.
/// 2. **Let go when answered.** Whoever keeps sources releases them
///    when done — a run once its executor has returned, the
///    container's id or the failure; a client's read scope once it
///    ends — through [`release`](Self::release): a source holds what
///    it fetched for a scope pinned until the scope ends, so a scope
///    left open would keep it. A source kept after the release is
///    cancelled at once.
/// 3. **A part is asked of the source that has it.** A manifest is
///    asked of every source at once, and the first to supply it does:
///    see [`manifest`]. What an image manifest names — its config and
///    its layers — is noted against the source that supplied that
///    manifest, and so is the manifest itself; a blob, and a proof
///    over that manifest, is asked of the source noted first, and of
///    the others in turn only where it does not answer: see [`blob`]
///    and [`prove`].
/// 4. **A miss is not a failure.** A source that answers a part not
///    held stays: with several, each holds only its own platforms. A
///    source whose scope fails is cancelled and forgotten.
pub struct Pulls {
    state: Mutex<State>,
}

/// What the lock protects. It is never held across an ask of a source.
struct State {
    /// The sources held, in the order they were kept.
    held: Vec<Arc<Source>>,
    /// Rule 3: a digest, to the source that supplied the manifest
    /// naming it.
    noted: HashMap<String, Arc<Source>>,
    /// Rule 2: released, and keeping nothing more.
    released: bool,
}

impl Pulls {
    /// No source yet.
    pub fn new() -> Self {
        Pulls {
            state: Mutex::new(State {
                held: Vec::new(),
                noted: HashMap::new(),
                released: false,
            }),
        }
    }

    /// Rule 2: every source cancelled and forgotten, and none kept
    /// from now on.
    pub async fn release(&self) {
        let held = {
            let mut state = self.state.lock().await;
            state.released = true;
            state.noted.clear();
            std::mem::take(&mut state.held)
        };
        for source in held {
            let _ = source.scope.cancel().await;
        }
    }

    /// Rule 1: `source` kept beside those kept already — `false`, and
    /// the source cancelled, once released (rule 2). A source kept
    /// already is kept once.
    pub async fn keep(&self, source: Arc<Source>) -> bool {
        let kept = {
            let mut state = self.state.lock().await;
            if state.released {
                false
            } else {
                if !state.held.iter().any(|held| Arc::ptr_eq(held, &source)) {
                    state.held.push(Arc::clone(&source));
                }
                true
            }
        };
        if !kept {
            let _ = source.scope.cancel().await;
        }
        kept
    }

    /// The first source held, if any: the one asked for what every
    /// source of an image holds alike — its document, its icons.
    pub async fn held(&self) -> Option<Arc<Source>> {
        self.state.lock().await.held.first().cloned()
    }

    /// Rule 3: every source held, the one `digest` is noted against
    /// first.
    async fn order(&self, digest: Option<&str>) -> Vec<Arc<Source>> {
        let state = self.state.lock().await;
        let noted = digest.and_then(|digest| state.noted.get(digest));
        let mut sources: Vec<Arc<Source>> = Vec::with_capacity(state.held.len());
        sources.extend(noted.filter(|noted| state.held.iter().any(|held| Arc::ptr_eq(held, noted))).cloned());
        sources.extend(state.held.iter().filter(|held| !noted.is_some_and(|noted| Arc::ptr_eq(held, noted))).cloned());
        sources
    }

    /// Rule 3: the manifest under `digest`, and what it names where it
    /// is an image manifest, noted against `source`. An index names
    /// manifests every source of the image may hold the index of, and
    /// notes nothing but itself.
    async fn note(&self, source: &Arc<Source>, digest: &str, body: &[u8]) {
        let named = named(body);
        let mut state = self.state.lock().await;
        if state.released {
            return;
        }
        state.noted.insert(digest.to_string(), Arc::clone(source));
        for digest in named {
            state.noted.entry(digest).or_insert_with(|| Arc::clone(source));
        }
    }

    /// Rule 4: `source` cancelled, unwaited, and forgotten where it is
    /// still held.
    async fn drop_source(&self, source: &Arc<Source>) {
        {
            let mut state = self.state.lock().await;
            state.held.retain(|held| !Arc::ptr_eq(held, source));
            state.noted.retain(|_, noted| !Arc::ptr_eq(noted, source));
        }
        let_go(Arc::clone(source));
    }
}

/// The digests an image manifest names — its config's and its layers'
/// — and none for an index, or for bytes that are no manifest.
fn named(body: &[u8]) -> Vec<String> {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return Vec::new();
    };
    let digest = |descriptor: &serde_json::Value| descriptor.get("digest").and_then(serde_json::Value::as_str).map(str::to_string);
    let mut digests: Vec<String> = value.get("config").and_then(digest).into_iter().collect();
    if let Some(layers) = value.get("layers").and_then(serde_json::Value::as_array) {
        digests.extend(layers.iter().filter_map(digest));
    }
    digests
}

/// The manifest under `digest` through the held sources, or `None`
/// when none is held or none answers it.
///
/// Every source is asked at once, and the first to supply the
/// manifest does; the asks still open are let go with it. One source
/// held is one ask. See rules 3 and 4 of [`Pulls`].
pub async fn manifest(pulls: &Pulls, digest: &str) -> Option<Manifest> {
    let mut asking: FuturesUnordered<_> = pulls
        .order(Some(digest))
        .await
        .into_iter()
        .map(|source| async move {
            let answered = source.scope.manifest(digest).await;
            (source, answered)
        })
        .collect();
    while let Some((source, answered)) = asking.next().await {
        match answered {
            Ok(Some(manifest)) => {
                pulls.note(&source, digest, &manifest.body).await;
                return Some(manifest);
            }
            Ok(None) => {}
            Err(_) => pulls.drop_source(&source).await,
        }
    }
    None
}

/// The blob under `digest` through the held sources, or `None` when
/// none is held or none answers it.
///
/// The source the digest is noted against is asked first, and the
/// others in turn where it does not hold it: rule 3 of [`Pulls`]. A
/// source is judged by its first piece: a channel that finishes with
/// none is a miss, and one that fails before any lets the source go
/// (rule 4). From the first piece on the stream is that source's,
/// ending short where it goes — the asking provider hashes what it got
/// and sees the shortfall, and nothing is retried mid-way.
pub async fn blob(pulls: &Pulls, digest: &str) -> Option<Blob> {
    for source in pulls.order(Some(digest)).await {
        let Ok(mut pieces) = source.scope.blob(digest).await else {
            pulls.drop_source(&source).await;
            continue;
        };
        match pieces.next().await {
            Some(Ok(first)) => {
                let rest = pieces.filter_map(|piece| ready(piece.ok()));
                return Some(Box::pin(stream::once(ready(first)).chain(rest)));
            }
            Some(Err(_)) => pulls.drop_source(&source).await,
            None => {}
        }
    }
    None
}

/// The proof of possession `request` names, over a held source's
/// image — see [`prove`](diverge_sdk::shared::containers::oci::prove)
/// — or `None` when none is held or none answers it.
///
/// The source that supplied the manifest the proof is over — the
/// request's platform — is asked first, and the others in turn where
/// it gives none: rule 3 of [`Pulls`]. The daemon hashes nothing
/// itself: the request is sent as it came, platform and nonce, on the
/// source's prove channel, and the proof answered is relayed as it
/// came. No image byte moves through the daemon for it.
pub async fn prove(pulls: &Pulls, request: &oci::prove::request::Request) -> Option<[u8; 32]> {
    for source in pulls.order(request.platform.as_deref()).await {
        match source.scope.prove(request).await {
            Ok(Some(proof)) => return Some(proof),
            Ok(None) => {}
            Err(_) => pulls.drop_source(&source).await,
        }
    }
    None
}
