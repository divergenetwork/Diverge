//! How Codex logs in: the method and its credential, as the caller
//! states them.

use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The login method, and with it the ONE credential the agent logs in
/// with.
///
/// Each method carries its credential as a member of its own: an
/// argument like any other, held from registration for the
/// container's life. The four are Codex's own ways in, as its source
/// has them (`rust-v0.163.0`, against the pinned `0.153.4`), and
/// nothing the run does with them is invented: a variable Codex
/// reads, or a file at the path Codex reads. No credential is empty:
/// the registration refuses one that is, naming the member and never
/// its value. [`fmt::Debug`] names the method and nothing else, so no
/// credential is ever formatted.
///
/// `OPENAI_API_KEY` in the environment is NOT one of them. Codex's
/// built-in `openai` provider has no `env_key` — the variable only
/// prefills the interactive login — so a key in the environment under
/// that name logs nothing in. `CODEX_API_KEY` is the variable the
/// credential store reads first, and `codex exec` honours it; the
/// `api_key` method is its key handed to Codex under that name.
///
/// A ChatGPT login — the `auth_file` method with ChatGPT tokens in
/// it — speaks only to OpenAI's own backend, and a
/// [`provider`](super::Agent::provider) beside one fails as itself,
/// in Codex's words; a provider's entry is keyed by `CODEX_API_KEY`,
/// so a provider REQUIRES `api_key`, and the registration refuses
/// the pair otherwise. Tagged `method`, snake case on the wire.
#[derive(Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum Auth {
    /// An OpenAI API key, put in the process environment as
    /// `CODEX_API_KEY`, the top-precedence credential — above
    /// `auth.json`, which is never written under this method. Bills
    /// the platform at API rates. The one method a
    /// [`provider`](super::Agent::provider) composes with.
    ApiKey {
        /// The key. Not empty.
        api_key: String,
    },
    /// A personal access token (`at-…`) or an Agent Identity JWT, put
    /// in the process environment as `CODEX_ACCESS_TOKEN`, where
    /// Codex's credential store reads it after `CODEX_API_KEY`.
    AccessToken {
        /// The token. Not empty.
        access_token: String,
    },
    /// Codex's own `auth.json`, whole, written to
    /// `$CODEX_HOME/auth.json` before every loop and read back after
    /// the loop's last turn. Codex rewrites the file in place when a
    /// ChatGPT login refreshes — the three tokens and `last_refresh`
    /// rotate, five minutes before the access token's expiry, eight
    /// days after `last_refresh`, or on a `401` — and what it leaves
    /// is kept in the agent's own `instance` database, so the next
    /// loop, and the next start of the container, finds the refreshed
    /// login: see [`login`](crate::login) for the rule. Bills
    /// whatever the login is.
    AuthFile {
        /// The file's content, a JSON object of any kind the file
        /// takes — ChatGPT tokens (`auth_mode`, `tokens {id_token,
        /// access_token, refresh_token, account_id}`, `last_refresh`),
        /// an `OPENAI_API_KEY` member, a `personal_access_token`, an
        /// `agent_identity`, Bedrock credentials. Not empty.
        auth: Map<String, Value>,
    },
    /// Workload identity federation: an identity token exchanged
    /// against a federation rule. The token is written to
    /// `$CODEX_HOME/identity-token` and named to Codex by
    /// `OPENAI_IDENTITY_TOKEN_FILE`, with `OPENAI_FEDERATION_RULE_ID`
    /// and, when given, `OPENAI_WORKLOAD_IDENTITY_CONTEXT` from the
    /// two members beside it.
    WorkloadIdentity {
        /// The identity token. Not empty.
        identity_token: String,
        /// The federation rule the token is exchanged against:
        /// `OPENAI_FEDERATION_RULE_ID`.
        federation_rule_id: String,
        /// The workload's context, when the rule wants one:
        /// `OPENAI_WORKLOAD_IDENTITY_CONTEXT`. Absent leaves the
        /// variable unset.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        context: Option<String>,
    },
}

impl Auth {
    /// The method's tag, as the wire spells it.
    pub fn method(&self) -> &'static str {
        match self {
            Auth::ApiKey { .. } => "api_key",
            Auth::AccessToken { .. } => "access_token",
            Auth::AuthFile { .. } => "auth_file",
            Auth::WorkloadIdentity { .. } => "workload_identity",
        }
    }

    /// The member the method's credential is, by name.
    pub fn member(&self) -> &'static str {
        match self {
            Auth::ApiKey { .. } => "api_key",
            Auth::AccessToken { .. } => "access_token",
            Auth::AuthFile { .. } => "auth",
            Auth::WorkloadIdentity { .. } => "identity_token",
        }
    }

    /// Whether the method's credential is empty: a string of no
    /// characters, or an object of no members.
    pub fn is_empty(&self) -> bool {
        match self {
            Auth::ApiKey { api_key } => api_key.is_empty(),
            Auth::AccessToken { access_token } => access_token.is_empty(),
            Auth::AuthFile { auth } => auth.is_empty(),
            Auth::WorkloadIdentity { identity_token, .. } => identity_token.is_empty(),
        }
    }
}

/// The method's tag, and no member: a credential is never formatted.
impl fmt::Debug for Auth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Auth").field("method", &self.method()).finish_non_exhaustive()
    }
}
