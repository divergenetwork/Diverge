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
///    itself, and picks where from, exactly as it does for a provider
///    that asks it for the image of a container it is to run: every
///    other provider connected that keeps a Store or lists a registry,
///    and every remote daemon on record, is asked at once whether it
///    can supply the image, by an
///    [`images::read`](crate::daemon::endpoints::images::read) scope
///    opened on it; the first that answers held supplies it, and every
///    other is let go. The provider pushed to is not asked: it has
///    looked already.
/// 4. **Judged as a run's image is.** The daemon asks as itself, of
///    every source it reaches, whatever the caller's grants are: a
///    push needs `push` over images reaching the provider pushed to,
///    and nothing more, whichever the source is.
/// 5. **A source that does not answer is not held.** A client that
///    does not answer, or a daemon that finds nobody holding the
///    image, or whose pick stops answering part way, is answered to
///    the provider as not held, and the push is then `Unavailable`,
///    or `ProofFailed`. No second pick is made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// The client that sent the request: `"client"`.
    Client,
    /// Whichever source the daemon finds: `"daemon"`.
    Daemon,
}
