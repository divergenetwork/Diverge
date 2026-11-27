//! What an agent or a tool is made from, less its name, its
//! provider and its mounts.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::daemon::endpoints::agents::create::client::request::Image;

/// Everything an agent or a tool is made from that is the same for
/// every one made from it, and that the image does not say for itself:
/// what it is for, the image, what it is given beyond what the image
/// needs, and the arguments.
/// What is not here is what differs one to
/// the next — the name, the provider it runs on, every mount of
/// providers' volumes — which the
/// [agent's](crate::daemon::endpoints::agents::create) or the
/// [tool's](crate::daemon::endpoints::standalone_tools::create) create
/// states; and what the image says of itself, in its
/// [document](crate::shared::containers::info::Document): the memory
/// and the disk it needs, and, of
/// a tool, how it is kept running. No template says any of those
/// again, so no two templates of one image disagree about the image.
///
/// # One shape, typed
///
/// `Type` is what the family says of itself, flat among the members —
/// [`AgentType`](crate::daemon::endpoints::agents::templates::AgentType),
/// which is `"type":"agent"` and nothing beside it, or
/// [`ToolType`](crate::daemon::endpoints::standalone_tools::templates::ToolType),
/// which is `"type":"tool"` and nothing beside it — so an agent
/// template is `{"type":"agent",…}` and a tool template
/// `{"type":"tool",…}`, neither decoding as the other, and the two
/// hashing apart however alike the rest.
///
/// # The image's kind is the template's
///
/// An agent template's image is one whose document's `type` is
/// `agent`. A tool template's image is one whose document's `type` is
/// `standalone_tool`: a `dependency_tool`'s image is made into a tool
/// by an agent's program that declares it, and by nothing else. A
/// template of an image of another kind is the create's error.
///
/// # No provider, so that it travels
///
/// A template is meant to be shared: handed from one caller to
/// another, published, made again anywhere and hashed the same. A
/// provider is one daemon's acquaintance, named by how that daemon
/// came to know it, and a template that named one would be that
/// daemon's alone. So the provider is the real agent's or tool's,
/// chosen at its create, and a template says nothing about where it
/// runs.
///
/// Its id is its hash: see [`template`](super). What a caller may
/// not choose is not here at all rather than here and ignored: the
/// container's name, its ports, its entrypoint and its environment
/// are the provider's. What a tool may ask of the daemon is not here
/// either: its program declares it at its registration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct Template<Type> {
    /// What the family says of itself, flat among the members: its
    /// `type`, `agent` or `tool`. Inside the hash, so the two kinds
    /// never hash the same.
    #[serde(flatten)]
    pub r#type: Type,
    /// The image: a digest and its references. See [`Image`]; the id
    /// hashes it with the references absent, see [`hashed`](Self::hashed).
    pub image: Image,
    /// How much memory the container has beyond what its image says
    /// it needs, in BYTES. `0` gives it what the image needs and no
    /// more.
    ///
    /// The container's limit is the image's own
    /// [`memory`](crate::shared::containers::info::Document::memory)
    /// and this, summed: a sum past 2 to the power 64 less 1 is the
    /// create's error. The limit is a ceiling, not a hint. A process
    /// that exceeds what the container is allowed is killed by the
    /// kernel rather than told — no failed allocation to catch, no
    /// warning first — and the container will not see the number in
    /// its own `/proc/meminfo`, which reports the host's. An image
    /// that sizes itself off what it thinks it has will size itself
    /// wrong.
    ///
    /// Bytes rather than megabytes because a unit that has to be
    /// spelled out in prose is a unit half of everyone gets wrong.
    pub extra_memory: u64,
    /// How much the container may WRITE beyond what its image says it
    /// needs, in BYTES. `0` gives it what the image needs and no more.
    ///
    /// The container's limit is the image's own
    /// [`disk`](crate::shared::containers::info::Document::disk) and
    /// this, summed, as [`extra_memory`](Self::extra_memory) is. The
    /// limit is on the container's own filesystem only — what it adds
    /// to or changes over the image it came from. The image's layers
    /// are read-only and are not counted, so a container starts at
    /// nothing however large the image is. It does not govern the
    /// mounts: a volume is storage that already existed, with a size
    /// of its own.
    ///
    /// Bytes rather than megabytes, for the reason
    /// [`extra_memory`](Self::extra_memory) gives.
    pub extra_disk: u64,
    /// What the image is told once, as the image defines it, for the
    /// container's life.
    ///
    /// A JSON value, because this crate does not know what an image
    /// takes — a model, tools, a tool server's own knobs — and a wire
    /// that typed it would have to be revised for every image that ever
    /// ran. It is handed to the container and not read here.
    pub arguments: Value,
}

impl<Type: Clone> Template<Type> {
    /// The template as its id hashes it: the same, with the image's
    /// references absent, since where the bytes may be fetched is not
    /// what the template is.
    pub fn hashed(&self) -> Template<Type> {
        Template {
            image: self.image.hashed(),
            ..self.clone()
        }
    }
}
