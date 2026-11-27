//! The Hermes agent.

use diverge_sdk::shared::containers::dependencies::Template;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Effort, Provider, Toolsets};

/// An agent running against Hermes (Nous Research's agent harness).
///
/// Every field here is HONORABLE: it names a mechanism the container
/// actually has — a `config.yaml` key its harness writes before
/// Hermes starts, or a per-request field on the run it drives — and
/// nothing else made the cut. What Hermes cannot be told through its
/// wire is not here: no skill preloading (skills arrive as mounts
/// under `/root/.hermes/external-skills/`, one directory per skill
/// holding its `SKILL.md`, and Hermes indexes them itself — that
/// path, not its own `skills/`, which it writes into), no
/// temperature (Hermes has no
/// main-agent sampling key), no thinking-token budget (the
/// [`effort`](Self::effort) ladder is the whole reasoning
/// vocabulary).
///
/// # Credentials
///
/// Every credential is a field: the provider's is a member of the
/// provider, and a toolset's a member of the toolset, each required
/// exactly where what it authenticates is chosen. A credential is
/// read from the arguments once, when the agent registers, and held
/// for the container's life; a login that refreshes is kept, as it
/// refreshes, in the program's own database. No error carries a
/// credential's value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Agent {
    /// The inference source — Hermes is multi-provider by design,
    /// this picks which, and carries the credential its auth
    /// needs. See [`Provider`].
    pub provider: Provider,
    /// The model to run, in the provider's own naming.
    pub model: String,
    /// The system prompt, applied per request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    /// Reasoning effort. Absent = the provider's own default;
    /// [`none`](Effort::None) = explicitly disabled — Hermes's own
    /// disable sentinel, which is why absence and `none` are
    /// different facts. See [`Effort`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<Effort>,
    /// Hermes's LOCAL capabilities, one switch each: an
    /// `Option<bool>` where a tool takes nothing, that tool's own
    /// argument structure where it takes
    /// anything — absent is off either way, and the whole structure
    /// absent is everything off. (The caller's MCP tools ride beside
    /// these regardless, and the tool containers the agent asks for
    /// are [`dependencies`](Self::dependencies).) See [`Toolsets`].
    #[serde(default)]
    pub toolsets: Toolsets,
    /// The tool containers this agent depends on, each a dependency
    /// tool template the caller deploys and serves to it as an MCP
    /// server, exactly as the SDK's [`Template`] states one: that
    /// type's JSON Schema is what `--schema` prints for this member,
    /// in `/.diverge/DIVERGE.md`, and this list, unchanged, is what the
    /// registration answers — one schema, from the one type, at both.
    /// Absent is none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<Template>,
}
