//! One setting of the provider's document, read or written by itself:
//! its port, its identity, whom it accepts, whom it dials, the
//! registries it pulls from. Every one keeps the rules of
//! [`setting`](super::setting).

use std::path::Path;

use diverge_provider::config::Config;
use diverge_provider::config::outgoing;
use diverge_sdk::shared;

use super::Error;
use super::setting::{add, delete, identities, line, number, read, set, write};
use crate::arguments::{Identity, Incoming, Outgoing, Podman, Port, Registries};
use crate::document::{self, Which};
use crate::lifecycle::Program;

/// `diverge provider port get|set`.
pub async fn port(root: &Path, action: Port) -> Result<(), Error> {
    match action {
        Port::Get => number(read::<Config>(root, Which::Provider).await?.port.into()).await,
        Port::Set { value } => {
            provider(root, |config| {
                config.port = value;
                Ok(())
            })
            .await
        }
    }
}

/// `diverge provider identity get|set`, by the rules of the daemon's
/// [`identity`](super::setting::identity): a `get` prints
/// `{"issuer":"…","subject":"…"}` or `null`; a `set` takes both the
/// issuer and the subject, writes the identity whole, and never makes
/// it absent.
pub async fn identity(root: &Path, action: Identity) -> Result<(), Error> {
    match action {
        Identity::Get => line(&read::<Config>(root, Which::Provider).await?.identity).await,
        Identity::Set { issuer, subject } => {
            provider(root, |config| {
                config.identity = Some(shared::Identity { issuer, subject });
                Ok(())
            })
            .await
        }
    }
}

/// `diverge provider incoming get|add|delete`: the list of identities
/// the provider accepts, `incoming`, as the daemon's `admins` is
/// worked on.
pub async fn incoming(root: &Path, action: Incoming) -> Result<(), Error> {
    match action {
        Incoming::Get => identities(&read::<Config>(root, Which::Provider).await?.incoming).await,
        Incoming::Add { issuer, subject } => {
            provider(root, |config| add("incoming", &mut config.incoming, shared::Identity { issuer, subject })).await
        }
        Incoming::Delete { index } => provider(root, |config| delete("incoming", &mut config.incoming, index)).await,
    }
}

/// `diverge provider outgoing get|add|delete`: the list of peers the
/// provider dials, `outgoing`, by these rules:
///
/// - A `get` prints every peer, one JSON line each, in the document's
///   order: `{"index":0,"address":"…","issuer":"…","subject":"…"}`,
///   the identity's two members beside the address and not beneath a
///   member of their own, as an `add` takes them.
/// - An `add` takes the address and the identity's issuer and
///   subject, all three, and puts the peer after the last. A peer the
///   list already has — the same address and the same identity — is
///   refused; the same identity at another address, or another
///   identity at the same address, is another peer.
/// - A `delete` takes the index.
pub async fn outgoing(root: &Path, action: Outgoing) -> Result<(), Error> {
    match action {
        Outgoing::Get => {
            let config = read::<Config>(root, Which::Provider).await?;
            let mut text = Vec::new();
            for (index, peer) in config.outgoing.iter().enumerate() {
                let line = serde_json::json!({
                    "index": index,
                    "address": peer.address,
                    "issuer": peer.identity.issuer,
                    "subject": peer.identity.subject,
                });
                serde_json::to_writer(&mut text, &line).map_err(document::Error::Json)?;
                text.push(b'\n');
            }
            write(text).await
        }
        Outgoing::Add { address, issuer, subject } => {
            provider(root, |config| {
                let identity = shared::Identity { issuer, subject };
                match config.outgoing.iter().position(|peer| peer.address == address && peer.identity == identity) {
                    Some(index) => {
                        Err(Error::Listed { list: "outgoing", item: format!("{identity}, at `{address}`,"), index })
                    }
                    None => {
                        config.outgoing.push(outgoing::Outgoing { address, identity });
                        Ok(())
                    }
                }
            })
            .await
        }
        Outgoing::Delete { index } => provider(root, |config| delete("outgoing", &mut config.outgoing, index)).await,
    }
}

/// `diverge provider podman …`: the settings of the `podman` section,
/// each named as the section names it.
pub async fn podman(root: &Path, action: Podman) -> Result<(), Error> {
    match action {
        Podman::Registries(action) => registries(root, action).await,
    }
}

/// `diverge provider podman registries get|add|delete`: the list of
/// registries the provider pulls from, `podman.registries`, by these
/// rules:
///
/// - A `get` prints every registry, one JSON line each, in the
///   document's order: `{"index":0,"host":"docker.io"}`. A document
///   with no `podman` section, or none of its `registries`, has the
///   default three, and those are what is printed.
/// - An `add` takes the host and puts it after the last, and refuses
///   one listed already. The host is written as given: that it is a
///   host, with no scheme and no path, is not asked here.
/// - A `delete` takes the index. Deleting the last registry leaves
///   the list empty, which is written as empty and is not the default
///   three.
/// - The list is read by a provider that uses podman, one with a
///   `containers` or an `images` section, and by no other; it is
///   written here either way.
async fn registries(root: &Path, action: Registries) -> Result<(), Error> {
    match action {
        Registries::Get => {
            let config = read::<Config>(root, Which::Provider).await?;
            let mut text = Vec::new();
            for (index, host) in config.podman.registries.iter().enumerate() {
                let line = serde_json::json!({"index": index, "host": host});
                serde_json::to_writer(&mut text, &line).map_err(document::Error::Json)?;
                text.push(b'\n');
            }
            write(text).await
        }
        Registries::Add { host } => {
            provider(root, |config| add("podman.registries", &mut config.podman.registries, host)).await
        }
        Registries::Delete { index } => {
            provider(root, |config| delete("podman.registries", &mut config.podman.registries, index)).await
        }
    }
}

/// A write to the provider's document: see
/// [`set`](super::setting::set).
async fn provider(root: &Path, change: impl FnOnce(&mut Config) -> Result<(), Error>) -> Result<(), Error> {
    set(root, Program::Provider, Which::Provider, change).await
}
