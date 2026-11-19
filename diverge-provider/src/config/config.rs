//! The document itself.


use diverge_sdk::container_proxy::outside::OUTSIDE_PORT;
use diverge_sdk::shared::Identity;
use serde::{Deserialize, Serialize};

use super::outgoing::Outgoing;
use super::containers::Containers;
use super::images::Images;
use super::messages::Messages;
use super::podman::Podman;
use super::volumes::Volumes;

/// The whole of `provider.yaml`: what the provider is told.
///
/// An absent file is this type's [`Default`], and the provider runs
/// on it: the proxy's own port, no identity of its own, no peer
/// admitted, no peer dialled, no containers, no volume, no Store, no
/// messages — and so no podman. Every section may be absent, meaning
/// what its doc says. An unknown key is an error, so a misspelled
/// setting is refused rather than ignored.
///
/// Every path inside the file resolves relative to the root, the
/// directory that holds `provider.yaml`, and never to the working
/// directory: [`load`](crate::config::load) resolves each as it
/// reads, so what a program holds is absolute.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    /// The TCP port the provider accepts the protocol on: a WebSocket
    /// upgrade at any path, on every interface. Absent means the port
    /// the container proxy listens on inside a container, `14979`,
    /// which nothing on the host takes: a container's proxy is
    /// published to a loopback port podman picks.
    #[serde(default = "proxy_port")]
    pub port: u16,
    /// Who the provider is: the one identity whose token it presents
    /// in every handshake — those it answers and those it makes —
    /// whatever issuer the peer's is from. It is an identity of the
    /// root's [`identities`](diverge_sdk::identities), signed in
    /// beforehand. It lists no issuer the provider trusts: whom the
    /// provider serves is `incoming`'s and `outgoing`'s to say, from
    /// whatever issuer each is. Absent, the provider has no identity, and completes no
    /// handshake.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<Identity>,
    /// The identities that may dial the provider: a peer that dials
    /// in and whose verified `(issuer, subject)` is not here is closed
    /// in the handshake. It says nothing of the peers the provider
    /// dials, each of which names its own. Absent, nobody dials in.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incoming: Vec<Identity>,
    /// The peers the provider dials, each with the identity that must
    /// answer: see [`Outgoing`]. Absent, it dials no one.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outgoing: Vec<Outgoing>,
    /// The podman the provider uses: its registries, its storage and
    /// its image cache. Read only where the provider uses podman at
    /// all — see [`podman_in_use`](Self::podman_in_use). Absent, or
    /// any field of it absent, means the defaults.
    #[serde(default)]
    pub podman: Podman,
    /// That the provider runs containers, and how much of the host
    /// they may take. Absent means it runs none: the `containers`
    /// capability is `false`, and every agents run and tools run is
    /// answered `Unsupported`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub containers: Option<Containers>,
    /// Where volumes may be created, and which exist already. Absent
    /// means no volume can be created and none exists: the provider
    /// keeps no volume mode, and every volume endpoint answers
    /// `Unsupported`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volumes: Option<Volumes>,
    /// Where the images clients build on the provider are kept, and
    /// how much they may take. Absent means no Store: a
    /// build is refused, a listing is empty, and no image is held for
    /// a caller but one a registry the provider uses serves.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub images: Option<Images>,
    /// The rate the provider relays direct messages at, between the
    /// identities connected to it. Absent means it relays none: a
    /// `daemons::listen` and a `daemons::message` are each answered
    /// unsupported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub messages: Option<Messages>,
    /// Whether the provider keeps its log: who connected, whether its
    /// handshake was taken, what it asked for and when it went, under
    /// `logs/` in the provider's own directory — see
    /// [`Log`](crate::log::Log). Absent means `true`. `false`, nothing
    /// is written.
    #[serde(default = "logging")]
    pub logging: bool,
}

/// What a provider runs on before it has written a line of
/// configuration: the proxy's port, no identity, nobody admitted,
/// nobody dialled, the [`Podman`] defaults — which nothing reads —
/// no containers, no volume, no Store, no messages.
impl Default for Config {
    fn default() -> Self {
        Config {
            port: proxy_port(),
            identity: None,
            incoming: Vec::new(),
            outgoing: Vec::new(),
            podman: Podman::default(),
            containers: None,
            volumes: None,
            images: None,
            messages: None,
            logging: logging(),
        }
    }
}

impl Config {
    /// Whether the provider uses podman at all, which decides whether
    /// it runs a single podman command, at its start or after:
    ///
    /// - a `containers` section: podman runs the containers;
    /// - an `images` section: podman builds an image into the Store,
    ///   and reads a file out of one for an `images::read`;
    /// - neither: no podman. Nothing the provider then does — its
    ///   volumes, its FUSE serves, its messages, its daemons — runs
    ///   it, the `podman` section is not read, and the host needs no
    ///   podman installed.
    pub fn podman_in_use(&self) -> bool {
        self.containers.is_some() || self.images.is_some()
    }
}

/// The port `port` takes when the file leaves it out.
/// What `logging` takes when the document leaves it out: `true`.
fn logging() -> bool {
    true
}

fn proxy_port() -> u16 {
    OUTSIDE_PORT
}
