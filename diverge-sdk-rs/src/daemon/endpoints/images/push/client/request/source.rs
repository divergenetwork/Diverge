//! Where a pushed image's bytes come from.

use serde::{Deserialize, Serialize};

/// Who answers the provider's asks for the image being pushed: its
/// manifests, its blobs and the proof that it is held. Snake case on
/// the wire: the string `"client"`, or the string `"daemon"`.
///
/// The rules:
///
/// 1. **Asked only when the provider asks.** A digest the daemon owns
///    on the provider already is `Exists`, and a reference naming a
///    registry the provider uses, which serves the digest, supplies
///    it: in neither case is the source asked anything, whichever it
///    is, and nothing is opened on it.
/// 2. **[`Client`](Self::Client).** The provider's asks are the
///    client's to answer: the daemon opens each on the client's scope
///    and carries the answer back, reading none of it.
/// 3. **[`Daemon`](Self::Daemon).** The daemon answers the asks
///    itself, and picks where from. Every provider connected that
///    keeps a Store or lists a registry, the one pushed to among them,
///    and every remote daemon on record, is asked at once which
///    platforms of the image it can supply, by an
///    [`images::read`](crate::daemon::endpoints::images::read) scope
///    opened on it. Each source that supplies a platform named which
///    no source before it did is kept for those platforms; every
///    other is let go, and the asking ends once every platform named
///    has a source.
/// 4. **Sources are combined.** An image whose platforms are spread
///    over several sources is pushed from all of them: one push to
///    the provider for each source, naming that source's platforms,
///    all at once. See [`push`](super::super::super).
/// 5. **Judged as a run's image is.** The daemon asks as itself, of
///    every source it reaches, whatever the caller's grants are: a
///    push needs `push` over images reaching the provider pushed to,
///    and nothing more, whichever the source is.
/// 6. **All found, or nothing moved.** A daemon that finds no source
///    for a platform named asks nothing of the provider, and the push
///    is `Unavailable`. A client that does not answer, or a source
///    that stops answering part way, is answered to the provider as
///    not held, and the push it was answering is then `Unavailable`,
///    or `ProofFailed`. No second source is asked for its platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// The client that sent the request: `"client"`.
    Client,
    /// Whichever source the daemon finds: `"daemon"`.
    Daemon,
}
