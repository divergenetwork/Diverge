//! The arguments, the template and the image, as the request that
//! made the container carried them.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::shared::containers::request::Image;

/// What the program is handed once: the `arguments` of the request
/// that made the container, typed to the same depth for the same
/// reason — a JSON value, because the image defines what it takes,
/// and what the value may be is the image's `/.diverge/DIVERGE.md`, which the
/// proxy reads and the program is never asked for — the
/// id of the template the container was made from, so that a program
/// knows which template it is an instance of — and the image the
/// container runs, so that a program knows which image it runs as.
/// The image is the provider's to supply: it is the `image` of the
/// request that made the container, which the provider deployed the
/// container from and carries to the proxy on its begin, and the
/// proxy hands it on here. No secret rides here:
/// the program declares which it takes in its answer, and their
/// values arrive on `POST /secrets` once the caller has them.
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
    /// The image the container runs: its manifest digest, and the
    /// references under which its bytes may be fetched, absent when
    /// there are none — the
    /// [`image`](crate::shared::containers::request::Container::image)
    /// of the request that made the container, verbatim, as the
    /// provider carried it.
    pub image: Image,
}
