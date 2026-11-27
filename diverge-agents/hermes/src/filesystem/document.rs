//! A login that refreshes, and where its document goes.

use serde_json::{Map, Value};

/// One login the agent's arguments carry that Hermes rewrites when
/// it refreshes a token: the argument it is, the document as the
/// arguments give it, and where the harness puts a document before
/// `hermes gateway` starts and reads it back from after the gateway
/// has exited.
///
/// The document that is put there is not always this one: see
/// [`credentials`](crate::credentials) for which is used.
#[derive(Clone, PartialEq, Eq)]
pub struct Document {
    /// The argument, by its path in the agent: `provider.oauth` or
    /// `toolsets.spotify.oauth`. It names the login in an error and
    /// in the program's database, and no value does.
    pub argument: &'static str,
    /// Where the document lands, and is read back from.
    pub target: Target,
    /// The document, as the arguments give it.
    pub login: Map<String, Value>,
}

/// Where a login's document is put.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// The `providers.<entry>` object of `auth.json`: written as the
    /// entry and read back from it.
    AuthEntry(&'static str),
    /// The Qwen CLI's own token file at the real home: written as
    /// the file and read back from it.
    QwenCreds,
}
