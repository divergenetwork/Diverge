//! `diverge run images merge`: one or more images made one image in
//! the daemon's Store on a provider, as
//! [`merge`](diverge_sdk::daemon::endpoints::images::merge) states it.

use clap::Args;
use diverge_sdk::daemon::endpoints::images::merge::{client::request, server::response};

use crate::run::{Below, Emitter, Error, Target, body, request_schema, response_schema, shape};

/// Merging images into one, on a provider.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(subcommand_negates_reqs = true, args_conflicts_with_subcommands = true)]
pub struct Arguments {
    /// What the request carries, as a JSON object: see
    /// [`request::Frame`] — the `provider` the image is made on; the
    /// and the `images` to merge, each a digest and the platforms
    /// taken of it; `relay` among
    /// its members has another daemon serve it.
    #[arg(value_name = "BODY")]
    pub body: Option<String>,
    /// What is beneath the request: `request-schema`, the JSON Schema
    /// of the body it takes, or `response-schema`, the JSON Schema of
    /// every line it prints — printed in place of putting any request.
    #[command(subcommand)]
    pub below: Option<Below>,
}

/// The request put, and every answer printed as it comes, to the
/// daemon's finish: a line for each blob pulled onto the provider and
/// each group of platforms that landed there, then the one line that
/// ends it — the image merged, with its digest, or why it was not. The
/// stream ends by itself, so nothing here cuts it.
pub async fn run(arguments: Arguments, target: &Target<'_>, emitter: &mut Emitter) -> Result<(), Error> {
    match arguments.below {
        Some(Below::RequestSchema(_)) => {
            return request_schema::<request::Frame>("diverge run images merge", emitter).await;
        }
        Some(Below::ResponseSchema(_)) => {
            return response_schema::<response::Frame>("diverge run images merge", emitter).await;
        }
        None => {}
    }
    let request: request::Frame = body(arguments.body.as_deref())?;
    let handle = target.dial().await?;
    shape::stream::<_, response::Frame, _>(&handle, &request, emitter, shape::never).await
}
