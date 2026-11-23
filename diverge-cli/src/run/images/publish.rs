//! `diverge run images publish`: one or more images sent to a registry
//! as one image, as
//! [`publish`](diverge_sdk::daemon::endpoints::images::publish) states
//! it.

use clap::Args;
use diverge_sdk::daemon::endpoints::images::publish::{client::request, server::response};

use crate::run::{Below, Emitter, Error, Target, body, request_schema, response_schema, shape};

/// Publishing images to a registry as one image.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(subcommand_negates_reqs = true, args_conflicts_with_subcommands = true)]
pub struct Arguments {
    /// What the request carries, as a JSON object: see
    /// [`request::Frame`] — the `images` to publish, each a digest and
    /// the platforms taken of it; the `reference` they go to; the
    /// `tags`; the `credential`; and the `provider` to
    /// do it on, which absent the daemon picks; `relay` among its
    /// members has another daemon serve it.
    #[arg(value_name = "BODY")]
    pub body: Option<String>,
    /// What is beneath the request: `request-schema`, the JSON Schema
    /// of the body it takes, or `response-schema`, the JSON Schema of
    /// every line it prints — printed in place of putting any request.
    #[command(subcommand)]
    pub below: Option<Below>,
}

/// The request put, and every answer printed as it comes, to the
/// daemon's finish: a line for each blob pulled onto the provider, each
/// group of platforms that landed there, the image merged and each
/// blob the registry took, then the one line that ends it — the image
/// published, with its digest and the provider it was done on, or why
/// it was not. The stream ends by itself, so nothing here cuts it.
pub async fn run(
    arguments: Arguments,
    target: &Target<'_>,
    emitter: &mut Emitter,
) -> Result<(), Error> {
    match arguments.below {
        Some(Below::RequestSchema(_)) => {
            return request_schema::<request::Frame>("diverge run images publish", emitter).await;
        }
        Some(Below::ResponseSchema(_)) => {
            return response_schema::<response::Frame>("diverge run images publish", emitter).await;
        }
        None => {}
    }
    let request: request::Frame = body(arguments.body.as_deref())?;
    let handle = target.dial().await?;
    shape::stream::<_, response::Frame, _>(&handle, &request, emitter, shape::never).await
}
