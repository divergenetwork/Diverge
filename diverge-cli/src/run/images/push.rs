//! `diverge run images push`: an image put into the daemon's Store on
//! a provider without building it, as
//! [`push`](diverge_sdk::daemon::endpoints::images::push) states it.

use std::sync::Arc;

use bytes::Bytes;
use clap::Args;
use diverge_sdk::daemon::endpoints::images::push::client::execute::execute;
use diverge_sdk::daemon::endpoints::images::push::client::request::Source;
use diverge_sdk::daemon::endpoints::images::push::{client::request, server::response};
use diverge_sdk::provider::client::{Manifest, OciStore};
use diverge_sdk::shared::containers::oci;
use diverge_sdk::shared::containers::request::Image;
use futures_util::StreamExt as _;
use futures_util::stream::Empty;

use crate::run::{Below, Emitter, Error, Target, body_with, request_schema, response_schema, shape};

/// Pushing an image into the daemon's Store on a provider.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(subcommand_negates_reqs = true, args_conflicts_with_subcommands = true)]
pub struct Arguments {
    /// What the request carries, as a JSON object: see
    /// [`request::Frame`] — the provider, the image by its digest and
    /// the references its bytes may be fetched under, the `source` its
    /// bytes come from — a provider or a daemon, and never `client` —
    /// and a description; `relay` among its members has another daemon
    /// serve it.
    #[arg(value_name = "BODY")]
    pub body: Option<String>,
    /// A platform to hold of the image, as `os/architecture`: `linux/amd64`,
    /// `linux/arm64`. Given more than once, each of them. Added to the
    /// body's `platforms`, which the request requires: at least one,
    /// here or in the body.
    #[arg(long = "platform", value_name = "OS/ARCH")]
    pub platforms: Vec<String>,
    /// What is beneath the request: `request-schema`, the JSON Schema
    /// of the body it takes, or `response-schema`, the JSON Schema of
    /// every line it prints — printed in place of putting any request.
    #[command(subcommand)]
    pub below: Option<Below>,
}

/// The request put, and every answer printed as it comes, to the
/// daemon's finish: a line for each blob pulled, then the one line
/// that ends it — the image pushed, with where it came from, or why
/// there is none. The rules:
///
/// 1. **This program holds no image**, so it is no source of one. A
///    request whose `source` is `client` is refused here, before any
///    daemon is dialled: the source is a provider or a daemon, which
///    the daemon reads the image from itself.
/// 2. **Nothing is asked of this program.** With any other source the
///    daemon opens no channel on this scope. Were one opened, it is
///    answered as not held, by [`Nothing`].
/// 3. The stream ends by itself, so nothing here cuts it.
pub async fn run(arguments: Arguments, target: &Target<'_>, emitter: &mut Emitter) -> Result<(), Error> {
    match arguments.below {
        Some(Below::RequestSchema(_)) => {
            return request_schema::<request::Frame>("diverge run images push", emitter).await;
        }
        Some(Below::ResponseSchema(_)) => {
            return response_schema::<response::Frame>("diverge run images push", emitter).await;
        }
        None => {}
    }
    let request: request::Frame = body_with(arguments.body.as_deref(), "platforms", arguments.platforms)?;
    // Rule 1.
    if request.source == Source::Client {
        return Err(Error::Unsupported("a push whose `source` is `client`: this program holds no image; name a provider or a daemon as the source"));
    }
    let handle = target.dial().await?;
    let mut answers = execute(&handle, &request, Arc::new(Nothing)).await.map_err(shape::exchange::open)?;
    while let Some(item) = answers.next().await {
        let answer = item.map_err(shape::exchange::stream)?;
        emitter.emit(&answer).await?;
    }
    Ok(())
}

/// The store of a client that holds no image: every ask is answered as
/// not held. Rule 2 of [`run`].
#[derive(Debug, Clone, Copy)]
struct Nothing;

impl OciStore for Nothing {
    async fn holds(&self, _image: &Image) -> Option<String> {
        None
    }

    type Blob = Empty<Bytes>;

    async fn manifest(&self, _image: &Image, _digest: &str) -> Option<Manifest> {
        None
    }

    async fn blob(&self, _image: &Image, _digest: &str) -> Option<Self::Blob> {
        None
    }

    async fn prove(&self, _image: &Image, _request: &oci::prove::request::Request) -> Option<[u8; 32]> {
        None
    }
}
