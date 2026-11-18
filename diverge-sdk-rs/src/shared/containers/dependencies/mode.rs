//! Which of the two modes a dependency template declares.

use serde::{Deserialize, Deserializer, Serialize};

use super::{Mount, Scope};

/// A lazy tool, or an eager one: the two things a dependency can be,
/// told apart on the wire by the `mode` member beside the template's
/// other members — `"lazy"` or `"eager"` — with the members of that
/// mode flat beside it, and no member of the other. Both say a
/// [`Scope`], which is how far the tool is shared: one per parent
/// agent, one per parent agent template, or one for all.
///
/// # A lazy tool runs while it is used
///
/// What that is depends on its scope:
///
/// 1. `instance`: the tool is the agent's run's. It is deployed when
///    the agent's container starts, from the template as the program
///    declared it, and stopped with the agent's run: it lives exactly
///    as long as one run of one agent, and the next run deploys it
///    again. It alone may mount the agent's own files and
///    directories, since there is one agent whose files they are, and
///    that agent serves them.
/// 2. `template` or `global`: the tool is one container for every
///    agent its scope covers. It is started for the first thing that
///    uses it — an exchange an attached agent sends it — and stopped
///    once the caller's idle time has passed with nothing using it,
///    as a lazy standalone tool is; the next use starts it again. No
///    agent's run is its run, so no run's end is its end, and it
///    mounts nothing of any agent's, since it is many agents'.
///
/// # An eager tool is kept while any agent may use it
///
/// Made once, when the first agent that declares it registers, under
/// the scope key its [`Scope`] names — the agent, the agent's
/// template, or the template alone — and kept by the caller from then
/// on: started again whenever its container ends, and at every start
/// of the caller's, for as long as any agent is attached to it. An
/// agent is attached by a registration that declares the template and
/// detached by one that no longer does, and an eager tool with no
/// agent attached goes down for good after the caller's idle time, an
/// attachment within that time keeping it. Nothing else makes an
/// eager tool inactive: it serves no agent's run in particular, so no
/// run's end is its end. It mounts nothing of any agent's.
///
/// # The database
///
/// In either mode the tool's database scope is shared as far as the
/// tool is, and lives as long as its scope's owner: the agent, the
/// agent template, or — for a global tool — the last agent template
/// whose agents ever attached.
///
/// # What does not decode
///
/// A lazy tool whose scope is `template` or `global` and which names
/// a mount: see [`Mode::check`].
#[derive(Debug, Clone, PartialEq, Serialize)]
#[derive(schemars::JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum Mode {
    /// A tool run while it is used: `"mode":"lazy"`.
    Lazy {
        /// How far the tool is shared, and its database with it: see
        /// [`Scope`]. Required.
        scope: Scope,
        /// Files of the agent container served live into the tool
        /// container, one each over FUSE: see [`Mount`]. The agent's
        /// path names a file; the tool's path is where it appears.
        /// Only where `scope` is `instance`. Absent when empty.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        fuse_file_mounts: Vec<Mount>,
        /// Directories of the agent container served live into the
        /// tool container, one each over FUSE: see [`Mount`]. The
        /// agent's path names a directory, whose whole tree the tool
        /// sees; the tool's path is where it appears. Only where
        /// `scope` is `instance`. Absent when empty.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        fuse_directory_mounts: Vec<Mount>,
    },
    /// A tool the caller keeps: `"mode":"eager"`.
    Eager {
        /// How far the tool is shared, and its database with it: see
        /// [`Scope`]. Required.
        scope: Scope,
    },
}

impl Mode {
    /// How far the tool is shared: the `scope` either mode says.
    pub fn scope(&self) -> Scope {
        match self {
            Mode::Lazy { scope, .. } | Mode::Eager { scope } => *scope,
        }
    }

    /// Whether the mode is one a caller deploys, and why not where it
    /// is not.
    ///
    /// The rules:
    ///
    /// 1. A lazy tool whose scope is `instance` may name mounts: the
    ///    one agent it is deployed for serves them.
    /// 2. A lazy tool whose scope is `template` or `global` names no
    ///    mount: it is many agents', and there is no one agent whose
    ///    files a mount would be.
    /// 3. An eager tool has no mounts to name, whatever its scope.
    pub fn check(&self) -> Result<(), &'static str> {
        match self {
            Mode::Lazy { scope, fuse_file_mounts, fuse_directory_mounts } => {
                if *scope != Scope::Instance && !(fuse_file_mounts.is_empty() && fuse_directory_mounts.is_empty()) {
                    return Err("a lazy tool mounts the agent's files only where its scope is `instance`");
                }
                Ok(())
            }
            Mode::Eager { .. } => Ok(()),
        }
    }
}

/// [`Mode`] as it is read off the wire, before [`Mode::check`]: the
/// same two variants with the same members.
#[derive(Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
enum Read {
    Lazy {
        scope: Scope,
        #[serde(default)]
        fuse_file_mounts: Vec<Mount>,
        #[serde(default)]
        fuse_directory_mounts: Vec<Mount>,
    },
    Eager {
        scope: Scope,
    },
}

/// Read as its members say, then held to [`Mode::check`]: a mode the
/// check refuses does not decode, so every party that reads a
/// template refuses the same ones.
impl<'de> Deserialize<'de> for Mode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mode = match Read::deserialize(deserializer)? {
            Read::Lazy { scope, fuse_file_mounts, fuse_directory_mounts } => Mode::Lazy { scope, fuse_file_mounts, fuse_directory_mounts },
            Read::Eager { scope } => Mode::Eager { scope },
        };
        mode.check().map_err(serde::de::Error::custom)?;
        Ok(mode)
    }
}
