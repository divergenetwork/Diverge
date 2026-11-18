//! What a provider does, of everything this specification lets a
//! provider do.

use serde::{Deserialize, Serialize};

use super::{ContainersCapabilities, ImagesCapabilities, MessagesCapabilities, VolumeCapabilities};
use crate::provider::endpoints::volumes::Mode;

/// Which of the specification's optional parts this provider serves.
///
/// Every provider serves every endpoint's wire — it reads every
/// request and answers it — but not every provider does everything a
/// request may ask: one runs no containers, one has no Store,
/// one pulls from no registry, one has no FUSE on its host, one keeps
/// volumes of some modes only, and keeps them more narrowly than the
/// mode allows.
/// Each such part is one member here, and a request past it is not
/// a failure but a refusal the endpoint has a variant of its own for,
/// named below member by member. A caller that read these composes
/// no such request; one that did not reads the refusal.
///
/// # Flat, and tolerant
///
/// One object, every member at the top, each an object of booleans
/// or an object of its own: `containers`, whose object names the
/// platforms run; `images`, whose object says whether there is a
/// Store, what the Store does and which registries are pulled from; the three volume modes, whose object says what
/// may be done with a volume of the mode; and `messages`, whose
/// object is its rate. A member absent is the capability
/// absent — `false`, the empty list, the object of every member
/// `false`, no object — so a provider states only what it does. A member a
/// reader does not know is ignored: see [`Frame`](super::Frame).
///
/// # No mode is no volumes
///
/// A provider with none of the three volume modes keeps no volumes,
/// and every endpoint of
/// [`volumes`](crate::provider::endpoints::volumes) — the eleven —
/// answers `Unsupported`: nothing is listed, nothing is made, and
/// there is no volume for a run or a build to name. [`volumes`](Self::volumes)
/// is that test.
///
/// # What is not here
///
/// The volume endpoints themselves, which every provider that keeps
/// volumes serves;
/// [`daemons`](crate::provider::endpoints::daemons), which is the
/// provider's configuration and refused as such; and every quantity
/// — how many containers, how much disk — which is a limit, not a
/// capability, and answered where it binds.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct Capabilities {
    /// The provider runs containers, and for which platforms: it
    /// answers
    /// [`containers::agents::run`](crate::provider::endpoints::containers::agents::run)
    /// and
    /// [`containers::tools::run`](crate::provider::endpoints::containers::tools::run).
    /// See [`ContainersCapabilities`]. Absent — no object — each
    /// answers `Unsupported`, the first and only response of its
    /// scope, and nothing is fetched and nothing is deployed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub containers: Option<ContainersCapabilities>,
    /// What the provider does with images: whether it keeps a Store,
    /// whether its Store builds, and the registries it pulls from. See
    /// [`ImagesCapabilities`]. Absent — the object of no member — it
    /// keeps no Store and pulls from no registry.
    #[serde(default)]
    pub images: ImagesCapabilities,
    /// What may be done with a volume in the
    /// [`Persistent`](Mode::Persistent) mode. Every member `false` —
    /// the object absent — the provider keeps no volume in the mode:
    /// no listed volume is in it, and a
    /// [`create`](crate::provider::endpoints::volumes::create) or an
    /// [`edit`](crate::provider::endpoints::volumes::edit) naming it
    /// is answered `ModeUnsupported`. See
    /// [`kept`](VolumeCapabilities::kept). A mode that is kept is not
    /// thereby one a volume may be created in: that is the object's
    /// own [`create`](VolumeCapabilities::create).
    #[serde(default)]
    pub volumes_persistent: VolumeCapabilities,
    /// What may be done with a volume in the
    /// [`Ephemeral`](Mode::Ephemeral) mode. Every member `false`, as
    /// for [`volumes_persistent`](Self::volumes_persistent).
    #[serde(default)]
    pub volumes_ephemeral: VolumeCapabilities,
    /// What may be done with a volume in the
    /// [`ReadOnly`](Mode::ReadOnly) mode. Every member `false`, as
    /// for [`volumes_persistent`](Self::volumes_persistent).
    #[serde(default)]
    pub volumes_read_only: VolumeCapabilities,
    /// The provider relays messages between the identities connected
    /// to it: it answers
    /// [`daemons::listen`](crate::provider::endpoints::daemons::listen)
    /// and [`daemons::message`](crate::provider::endpoints::daemons::message),
    /// at no more than this rate. Absent, it relays none, and each of
    /// the two answers `Unsupported`, the first and only response of
    /// its scope. See [`MessagesCapabilities`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub messages: Option<MessagesCapabilities>,
}

impl Capabilities {
    /// Whether the provider keeps volumes at all: any mode among the
    /// three. `false` is every volume endpoint answering
    /// `Unsupported`.
    pub fn volumes(&self) -> bool {
        self.volumes_persistent.kept() || self.volumes_ephemeral.kept() || self.volumes_read_only.kept()
    }

    /// What may be done with a volume of `mode` here: `None` for a
    /// mode the provider keeps no volumes in — its object every
    /// member `false`.
    pub fn volume(&self, mode: Mode) -> Option<&VolumeCapabilities> {
        let object = match mode {
            Mode::Persistent => &self.volumes_persistent,
            Mode::Ephemeral => &self.volumes_ephemeral,
            Mode::ReadOnly => &self.volumes_read_only,
        };
        object.kept().then_some(object)
    }
}
