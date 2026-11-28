//! The arguments, the template, which record of it the container is,
//! and the image, as the request that made the container carried
//! them.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::shared::containers::request::{Image, Which};

/// What the program is handed once: the `arguments` of the request
/// that made the container, typed to the same depth for the same
/// reason — a JSON value, because the image defines what it takes,
/// and what the value may be is the image's `/.diverge/DIVERGE.md`, which the
/// proxy reads and the program is never asked for — the
/// id of the template the container was made from, so that a program
/// knows which template it is an instance of — which record of that
/// template it is, so that a program can name itself, and what is its
/// own, in whatever it asks of the caller — and the image the
/// container runs, so that a program knows which image it runs as.
/// The image is the provider's to supply: it is the `image` of the
/// request that made the container, which the provider deployed the
/// container from and carries to the proxy on its begin, and the
/// proxy hands it on here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    /// The arguments, as the image defines them.
    pub arguments: Value,
    /// The template the container was made from, by id: an agent's
    /// agent template, a tool's tool template, or a dependency's
    /// dependency tool template — each the lowercase hexadecimal
    /// SHA-256 of the template's canonical bytes, as the caller that
    /// made the container names it.
    pub template: String,
    /// Which record of that template the container is, flat beside
    /// `template`: an agent's or a standalone tool's `index`, or a
    /// dependency tool's `owner`. With the template it is the record's
    /// own name: an agent names its own `instance` dependency tool by
    /// the dependency's template and the owner
    /// `{"agent":{"template":…,"index":…}}` that these two members
    /// make. See [`Which`].
    #[serde(flatten)]
    pub which: Which,
    /// The image the container runs: its manifest digest, and the
    /// references under which its bytes may be fetched, absent when
    /// there are none — the
    /// [`image`](crate::shared::containers::request::Container::image)
    /// of the request that made the container, verbatim, as the
    /// provider carried it.
    pub image: Image,
}
