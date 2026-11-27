//! Web search and page extraction.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Web search and page extraction, with its arguments. The field
/// being absent from [`Toolsets`](super::Toolsets) is Hermes's own
/// default for this toolset; present is the switch thrown on.
///
/// Two roles, two slots: [`search`](Self::search) is what answers
/// queries, [`extract`](Self::extract) is what turns a page into
/// text — different tools, different backends, independently
/// keyed. Each slot names ONE backend, and a keyed one carries its
/// key beside its name; a slot left absent runs on Hermes's keyless
/// free-tier ring, which needs nothing — the keys buy quality and
/// quota, not existence. Every combination is coherent, the empty
/// object included.
#[derive(
    Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema,
)]
pub struct Toolset {
    /// The search backend, one of [`Search`]'s; absent = the
    /// keyless ring.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search: Option<Search>,
    /// The extraction backend, as [`Extract`] names it; absent = the
    /// keyless ring.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extract: Option<Extract>,
}

/// One search backend. Untagged — a keyed backend is an object
/// carrying its name and its key, a self-hosted SearXNG is an object
/// carrying its address — and the two share no member, so no marker
/// is needed.
///
/// One backend, deliberately: Hermes can pool several keys as
/// fallbacks for each other, but a vocabulary where extra keys
/// quietly change behavior is the environment-dependent meaning
/// this module keeps rejecting. The caller names the backend; the
/// keyless ring remains the fallback story. The harness applies the
/// keyed backend's key in the gateway's process environment, or
/// SearXNG's address, and pins `web.search_backend` to the named
/// provider in the config it owns.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Search {
    /// A keyed backend, by name, with its key.
    Keyed {
        /// The backend: see [`Keyed`].
        backend: Keyed,
        /// The backend's API key, applied under the variable
        /// [`Keyed`] names for it.
        api_key: String,
    },
    /// A self-hosted SearXNG instance, applied as `SEARXNG_URL` —
    /// the one backend whose argument is an address, and no key.
    Searxng {
        /// The instance.
        searxng_url: String,
    },
}

/// The keyed search backends, by Hermes's own plugin names.
/// Kebab-case on the wire. Each variant says the variable its key is
/// applied under, in the gateway's process environment.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum Keyed {
    /// Tavily. Its key is applied as `TAVILY_API_KEY`.
    Tavily,
    /// Exa. Its key is applied as `EXA_API_KEY`.
    Exa,
    /// Parallel. Its key is applied as `PARALLEL_API_KEY`.
    Parallel,
    /// Keenable. Its key is applied as `KEENABLE_API_KEY`.
    Keenable,
    /// Brave, on its free plan. Its key is applied as
    /// `BRAVE_SEARCH_API_KEY`.
    BraveFree,
}

/// The extraction backend, with its key. The harness applies the
/// key in the gateway's process environment and pins
/// `web.extract_backend` to the named backend.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct Extract {
    /// The backend: see [`ExtractBackend`].
    pub backend: ExtractBackend,
    /// The backend's API key, applied under the variable
    /// [`ExtractBackend`] names for it.
    pub api_key: String,
}

/// The keyed extraction backends, by Hermes's own plugin names —
/// single-variant today because Hermes has one keyed extractor at
/// this pin. Kebab-case on the wire.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum ExtractBackend {
    /// Firecrawl. Its key is applied as `FIRECRAWL_API_KEY`.
    Firecrawl,
}
