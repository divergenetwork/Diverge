//! Provider definitions — the inference source, and the credential
//! it takes.
//!
//! AUTH IS AN ARGUMENT. Each provider here is a structure carrying
//! its discriminator, the endpoint facts that provider needs, and
//! its credential, a member of its own: `api_key` wherever the
//! provider takes one key, and otherwise a member named for what it
//! is — [`copilot`]'s `github_token`, [`vertex`]'s
//! `service_account_json`, [`bedrock`]'s `auth`, whose members follow
//! from the shape it names, and `oauth` for the four logins that
//! refresh. A credential is required exactly where its provider is
//! chosen; [`custom`]'s key alone is optional, and
//! [`opencode_free`] takes none. No error, anywhere, carries a
//! credential's value: a failure names the member.
//!
//! HOW THE HARNESS APPLIES IT: each structure's doc names the
//! mechanism, and all but two reduce to setting the provider's
//! canonical env var(s) in the gateway's PROCESS environment before
//! Hermes starts, to the member's value. The
//! two: [`custom`]'s key goes into `model.api_key` of the config,
//! the only place Hermes reads a custom key from; [`vertex`]'s
//! document goes to the file `VERTEX_CREDENTIALS_PATH` names.
//! Process env is load-bearing twice over: Hermes prefers
//! `~/.hermes/.env` OVER process env when that file exists — so the
//! image must guarantee it does not — and bedrock's botocore chain
//! reads only real env, never Hermes's dotenv.
//!
//! AN OAUTH LOGIN REFRESHES. Four providers — [`nous`],
//! [`openai_codex`], [`minimax_oauth`], [`qwen_oauth`] —
//! authenticate with OAuth state whose refresh tokens are
//! SINGLE-USE: every refresh rotates them, and whoever holds the
//! old copy is logged out. The arguments never change, so the
//! argument alone cannot carry that: these structures take their
//! state document as `oauth`, a JSON object the harness writes where
//! Hermes reads it before the gateway starts and reads back as the
//! run left it after the gateway exits, and the program keeps what it
//! read back in its own database, for the next run:
//! [`credentials`](crate::credentials) states the rule by which the
//! kept document or the argument is used.
//! (These providers' advertised key env vars — `NOUS_API_KEY`,
//! `QWEN_API_KEY` — are vestigial; no resolution path reads them, so
//! the state document is the only credential there is.)
//!
//! DROPPED from Hermes's bundled registry: `copilot-acp` alone,
//! which keeps its auth inside a spawned external CLI, not in
//! Hermes at all. Keyed Copilot remains as [`copilot`].

pub mod actual;
pub mod ai_gateway;
pub mod alibaba;
pub mod alibaba_coding_plan;
pub mod anthropic;
pub mod arcee;
pub mod azure_foundry;
pub mod bedrock;
pub mod commandcode;
pub mod commandcode_anthropic;
pub mod copilot;
pub mod custom;
pub mod deepinfra;
pub mod deepseek;
pub mod fireworks;
pub mod gemini;
pub mod gmi;
pub mod huggingface;
pub mod kilocode;
pub mod kimi_coding;
pub mod kimi_coding_cn;
pub mod meta_ai;
pub mod minimax;
pub mod minimax_cn;
pub mod minimax_oauth;
pub mod nebius_token_factory;
pub mod nous;
pub mod novita;
pub mod nvidia;
pub mod ollama_cloud;
pub mod openai_codex;
pub mod opencode_free;
pub mod opencode_go;
pub mod opencode_zen;
pub mod openrouter;
pub mod qwen_oauth;
pub mod router;
pub mod stepfun;
pub mod upstage;
pub mod vertex;
pub mod xai;
pub mod xiaomi;
pub mod zai;

mod provider;

pub use provider::*;
