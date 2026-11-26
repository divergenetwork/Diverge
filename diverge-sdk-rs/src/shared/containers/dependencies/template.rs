//! One dependency, as an agent's program declares it.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::Mount;
use super::super::request::Image;

/// One tool container an agent's program depends on: everything the
/// caller needs to deploy it that the tool's image does not say for
/// itself, as fields and not as words. A tool container depends on
/// nothing.
///
/// A [`Container`](crate::shared::containers::request::Container) is an
/// image, two limits, mounts and arguments. The image and the
/// arguments are here, member for member. The limits are the image's
/// own — the `memory` and the `disk` of its
/// [document](crate::shared::containers::info::Document) — and what
/// this adds to each. The mounts are not the caller's volumes — a
/// volume is named in the caller's own listing, and a tool that named
/// one would be dictating the caller's storage — but the AGENT's own
/// files and directories, each a path in the agent container served
/// live at a path in the tool container: the caller serves the agent
/// container, a `containers::fuse_serve` of the path, and mounts the
/// tool, a `fuse::mount` answered from it.
///
/// # What the image says, and this does not
///
/// The image is a dependency tool's: its document's `type` is
/// `dependency_tool`, and a declaration of an image of another kind —
/// an agent's, or a standalone tool's — is the registration's error.
/// That document, and not this, says:
///
/// 1. How the tool is kept running, its
///    [`mode`](crate::shared::containers::info::Mode): lazy or eager.
/// 2. How far it is shared, its
///    [`scope`](crate::shared::containers::info::Scope): one per
///    agent, or one for all.
/// 3. Every secret its program could take, each by its name and what
///    it is. Which of them this tool needs is here, by name: see
///    [`secrets`](Self::secrets).
/// 4. The memory and the disk it needs.
///
/// So two agents that declare one image declare one kind of tool, and
/// no declaration says of an image what the image does not.
///
/// # The mounts are a lazy tool's of one agent
///
/// A mount is the declaring agent's own path, so there must be one
/// agent whose path it is, and that agent's run must be the tool's:
/// only an image whose `mode` is `lazy` and whose `scope` is
/// `instance` is declared with any. A declaration that names a mount
/// of an image of any other mode or scope is the registration's
/// error.
///
/// What the tool may ask of the caller is not here: the tool's own
/// program declares it at its registration, every time it starts, as a
/// standalone tool's does — see the `permissions` of the
/// [`register`](crate::container_proxy::inside::register) answer.
///
/// There are no instructions. What a declaration cannot say — the
/// provider the tool runs on, the volumes it is given — is the
/// caller's, and the caller needs no prose to choose it. The daemon's
/// own tool templates are another thing: a record the daemon keeps,
/// named by a hash, made by a request, of a standalone tool's image;
/// this is what an agent's program declares and a caller deploys.
///
/// There is no name. A dependency is named by its ID: the lowercase
/// hexadecimal SHA-256 of its [`canonical`](crate::shared::canonical)
/// bytes — this template as compact JSON, absent members omitted, the
/// image's `references` absent too, every object key sorted at every
/// depth, the `arguments`, the mounts and the `secrets` included, the
/// names sorted bytewise — sixty-four characters, which the caller computes over
/// [`hashed`](Self::hashed). Two declarations with one id are one
/// dependency, and a list that declares one twice is refused; the same
/// template deployed for the same agent, on whatever provider, is the
/// same dependency, and the same template declared under the same
/// scope key, by whatever agent, is the same tool. The caller labels
/// the tool's MCP tools by the id when it merges lists, and the
/// program knows which tool serves each by the keys under `_meta`, see
/// [`shared::mcp`](crate::shared::mcp).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
pub struct Template {
    /// The image: a digest and its references, as a
    /// `containers::tools::run` request names one. A dependency tool's.
    pub image: Image,
    /// The memory the tool container has beyond what its image says it
    /// needs, in bytes. `0` gives it what the image needs and no more.
    /// The container's limit is the image's own `memory` and this,
    /// summed: a sum past 2 to the power 64 less 1 is the
    /// registration's error.
    pub extra_memory: u64,
    /// The bytes the tool container may write beyond what its image
    /// says it needs. `0` gives it what the image needs and no more.
    /// The container's limit is the image's own `disk` and this,
    /// summed, as [`extra_memory`](Self::extra_memory) is.
    pub extra_disk: u64,
    /// The arguments the tool container is registered with — the
    /// `arguments` of the run request that makes it, verbatim.
    pub arguments: Value,
    /// The secrets the tool container needs, by name: all of those its
    /// image's document states, or some of them, and none it does not
    /// state. Required, and empty for a tool that needs none. Which of
    /// them the tool's program takes, and in what mode, is its own
    /// answer at its registration — see
    /// [`secrets`](crate::shared::containers::secrets) — and the value
    /// of each is resolved through the vault key the agent the tool is
    /// spawned for binds the same name to, fed at every activation.
    /// Inside the hash, sorted bytewise there. A name is not empty, no
    /// two are alike, every one is a name the tool's image states, and
    /// every one is a name the registering agent binds; a declaration
    /// that says otherwise is the registration's error.
    pub secrets: Vec<String>,
    /// Files of the agent container served live into the tool
    /// container, one each over FUSE: see [`Mount`]. The agent's path
    /// names a file; the tool's path is where it appears. Only of an
    /// image whose `mode` is `lazy` and whose `scope` is `instance`.
    /// Absent when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fuse_file_mounts: Vec<Mount>,
    /// Directories of the agent container served live into the tool
    /// container, one each over FUSE: see [`Mount`]. The agent's path
    /// names a directory, whose whole tree the tool sees; the tool's
    /// path is where it appears. Only of an image whose `mode` is
    /// `lazy` and whose `scope` is `instance`. Absent when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fuse_directory_mounts: Vec<Mount>,
}

impl Template {
    /// The template as its id hashes it: the same, with the image's
    /// references absent, since where the bytes may be fetched is not
    /// what the tool is, and the secrets' names sorted bytewise, since
    /// the order they are named in is not what is needed.
    pub fn hashed(&self) -> Template {
        let mut secrets = self.secrets.clone();
        secrets.sort_unstable();
        Template {
            image: self.image.hashed(),
            secrets,
            ..self.clone()
        }
    }

    /// Whether the template names any mount of the agent's files or
    /// directories: what only a lazy tool of one agent may.
    pub fn mounts(&self) -> bool {
        !(self.fuse_file_mounts.is_empty() && self.fuse_directory_mounts.is_empty())
    }
}
