//! The login: the agent's method, applied with its credential.
//!
//! The agent names the method and carries its credential
//! ([`Auth`]); at each loop's start [`Auth::resolve`] turns them into
//! what the run needs — the variables the process gets, and a file to
//! write under the home when the method wants one — without dialing
//! anything. Under `auth_file` the file's content is the login the
//! run was handed: the argument, or what the last loop left of it,
//! as [`login`](crate::login) decides.
//!
//! What goes where is Codex's own, not this crate's: `CODEX_API_KEY`
//! and `CODEX_ACCESS_TOKEN` are the variables its credential store
//! reads, in that order and above `auth.json`; `auth.json` under
//! `CODEX_HOME` is the file it reads and, on a ChatGPT refresh,
//! rewrites in place; `OPENAI_IDENTITY_TOKEN_FILE` names the token a
//! workload identity login exchanges. `OPENAI_API_KEY` in the
//! environment is none of these: the built-in provider has no
//! `env_key`, so the `api_key` method hands its key to Codex as
//! `CODEX_API_KEY`.

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::agent::Auth;

/// Codex's home, fixed for the container's life: its default for the
/// root user, set explicitly on the process so the geometry never
/// moves.
pub const CODEX_HOME: &str = "/root/.codex";

/// The name of Codex's login file, under the home.
pub const AUTH_FILE: &str = "auth.json";

/// The name of the workload identity token's file, under the home.
pub const IDENTITY_TOKEN_FILE: &str = "identity-token";

/// The variable the `access_token` method's token is handed to Codex
/// under: a personal access token or an Agent Identity JWT.
pub const CODEX_ACCESS_TOKEN: &str = "CODEX_ACCESS_TOKEN";

/// The variable Codex's credential store reads first: the API key.
pub const CODEX_API_KEY: &str = "CODEX_API_KEY";

/// The variable naming the file the identity token is read from.
pub const OPENAI_IDENTITY_TOKEN_FILE: &str = "OPENAI_IDENTITY_TOKEN_FILE";

/// The variable naming the federation rule the token is exchanged
/// against.
pub const OPENAI_FEDERATION_RULE_ID: &str = "OPENAI_FEDERATION_RULE_ID";

/// The variable carrying the workload's context, when the rule wants
/// one.
pub const OPENAI_WORKLOAD_IDENTITY_CONTEXT: &str = "OPENAI_WORKLOAD_IDENTITY_CONTEXT";

/// Where Codex keeps its login: [`AUTH_FILE`] under [`CODEX_HOME`].
pub fn auth_file() -> PathBuf {
    Path::new(CODEX_HOME).join(AUTH_FILE)
}

/// Where the identity token is written: [`IDENTITY_TOKEN_FILE`]
/// under [`CODEX_HOME`].
pub fn identity_token_file() -> PathBuf {
    Path::new(CODEX_HOME).join(IDENTITY_TOKEN_FILE)
}

/// The login, ready for the process: what [`Auth::resolve`] made of
/// the method and its credential.
pub struct Resolved {
    /// The variables the process gets, beside `CODEX_HOME`.
    pub env: BTreeMap<String, String>,
    /// A file to write under the home before the first turn, when
    /// the method wants one: `auth.json`, or the identity token.
    pub file: Option<(PathBuf, Vec<u8>)>,
}

impl Resolved {
    /// Write the file, when there is one, the home made as needed.
    pub async fn write(&self) -> io::Result<()> {
        let Some((path, bytes)) = &self.file else {
            return Ok(());
        };
        tokio::fs::create_dir_all(CODEX_HOME).await?;
        tokio::fs::write(path, bytes).await
    }
}

impl Auth {
    /// The method applied: its credential placed where Codex reads
    /// it. `kept` is the `auth_file` method's login as
    /// [`login::current`](crate::login::current) answered it, and is
    /// written in place of the argument when there is one; the other
    /// methods ignore it. The one failure is `auth.json` that will
    /// not serialize, which names nothing of its content.
    pub fn resolve(&self, kept: Option<&Map<String, Value>>) -> Result<Resolved, serde_json::Error> {
        let mut env = BTreeMap::new();
        let mut file = None;
        match self {
            Auth::ApiKey { api_key } => {
                env.insert(CODEX_API_KEY.to_string(), api_key.clone());
            }
            Auth::AccessToken { access_token } => {
                env.insert(CODEX_ACCESS_TOKEN.to_string(), access_token.clone());
            }
            Auth::AuthFile { auth } => {
                let bytes = serde_json::to_vec(kept.unwrap_or(auth))?;
                file = Some((auth_file(), bytes));
            }
            Auth::WorkloadIdentity {
                identity_token,
                federation_rule_id,
                context,
            } => {
                let path = identity_token_file();
                env.insert(
                    OPENAI_IDENTITY_TOKEN_FILE.to_string(),
                    path.to_string_lossy().into_owned(),
                );
                env.insert(OPENAI_FEDERATION_RULE_ID.to_string(), federation_rule_id.clone());
                if let Some(context) = context {
                    env.insert(OPENAI_WORKLOAD_IDENTITY_CONTEXT.to_string(), context.clone());
                }
                file = Some((path, identity_token.clone().into_bytes()));
            }
        }
        Ok(Resolved { env, file })
    }

    /// What the loop leaves of its login, read back after the last
    /// turn: for `auth_file`, `auth.json` as Codex left it. `None`
    /// for the other methods, which leave nothing, and for a file
    /// that cannot be read or is not a JSON object — what is kept
    /// then stands.
    pub async fn read_back(&self) -> Option<Map<String, Value>> {
        match self {
            Auth::AuthFile { .. } => {
                let bytes = tokio::fs::read(auth_file()).await.ok()?;
                serde_json::from_slice::<Map<String, Value>>(&bytes).ok()
            }
            Auth::ApiKey { .. } | Auth::AccessToken { .. } | Auth::WorkloadIdentity { .. } => None,
        }
    }
}
