//! One setting of the provider's document, read or written by itself:
//! its port, its identity, whom it accepts, whom it dials. Every one
//! keeps the rules of [`setting`](super::setting).

use std::path::Path;

use diverge_provider::config::Config;
use diverge_provider::config::outgoing;
use diverge_sdk::shared;

use super::Error;
use super::setting::{add, delete, identities, line, number, read, set, write};
use crate::arguments::{Identity, Incoming, Outgoing, Port};
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

/// A write to the provider's document: see
/// [`set`](super::setting::set).
async fn provider(root: &Path, change: impl FnOnce(&mut Config) -> Result<(), Error>) -> Result<(), Error> {
    set(root, Program::Provider, Which::Provider, change).await
}
