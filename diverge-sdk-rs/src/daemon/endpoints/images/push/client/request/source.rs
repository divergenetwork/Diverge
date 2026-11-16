//! Where a pushed image's bytes come from.

use serde::{Deserialize, Serialize};

use crate::daemon::agent_log::Identity;
use crate::shared::relay::Relay;

/// Who answers the provider's asks for the image being pushed: its
/// manifests, its blobs and the proof that it is held. Snake case on
/// the wire: the string `"client"`, or an object of exactly one
/// member, `provider` or `daemon`.
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
/// 3. **[`Provider`](Self::Provider).** The daemon answers the asks
///    itself, from an
///    [`images::read`](crate::provider::endpoints::images::read) scope
///    it opens, as itself, on the provider named: one it keeps a
///    record of and is connected to. So the image comes from the
///    daemon's own Store there, or from a registry that provider
///    uses. The provider named may be the one pushed to.
/// 4. **[`Daemon`](Self::Daemon).** The daemon answers the asks itself,
///    from an
///    [`images::read`](crate::daemon::endpoints::images::read) scope it
///    opens on a remote daemon. The route is read exactly as a
///    request's [`relay`](Relay) is: the first ID is this daemon's
///    record of a remote daemon, on which the scope is opened as the
///    account that record names, and the IDs after it are that
///    request's own `relay`, so the image is read on the daemon the
///    route ends at. A route of no ID names no daemon, and the request
///    is refused as an error.
/// 5. **Judged as what it is.** A provider named as the source is
///    reached by the `read` action of the `providers` grant, as an
///    [`images::read`](crate::daemon::endpoints::images::read) reaches
///    one; a route by the `relay` action of the `daemons` grant over
///    its first record, as every relay is. See
///    [`grant`](crate::daemon::grant). The client needs no grant to be
///    asked for what it sends.
/// 6. **A source that does not answer is not held.** A source that
///    does not hold the image, that cannot be reached, or that stops
///    answering part way, is answered to the provider as not held, and
///    the push is then `Unavailable`, or `ProofFailed`. No other
///    source is tried.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// The client that sent the request: `"client"`.
    Client,
    /// One provider on the daemon's record, by its identity:
    /// `{"provider":{…}}`.
    Provider(Identity),
    /// One remote daemon, by the route to it: `{"daemon":[…]}`.
    Daemon(Relay),
}
