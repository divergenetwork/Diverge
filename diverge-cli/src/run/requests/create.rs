//! `diverge run requests create`: a request made of others, waited on
//! to its answer, as
//! [`create`](diverge_sdk::daemon::endpoints::requests::create) states
//! it.

use clap::Args;
use diverge_sdk::daemon::endpoints::requests::create::{client::request, server::response};

use crate::run::{Below, Emitter, Error, Target, body, request_schema, response_schema, shape};

/// Making a request of accounts and tools.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(subcommand_negates_reqs = true, args_conflicts_with_subcommands = true)]
pub struct Arguments {
    /// What the request carries, as a JSON object: see
    /// [`request::Frame`] — `request`, content blocks as a chat
    /// message's are, which every listener is sent as they are; and `to`, whom it is made of: the
    /// string `all`, or `{"only": {…}}` naming accounts, standalone
    /// tools and dependency tools, each by itself or by a tag; `relay`
    /// among its members has another daemon serve it.
    #[arg(value_name = "BODY")]
    pub body: Option<String>,
    /// What is beneath the request: `request-schema`, the JSON Schema
    /// of the body it takes, or `response-schema`, the JSON Schema of
    /// every line it prints — printed in place of putting any request.
    #[command(subcommand)]
    pub below: Option<Below>,
}

/// The request put, and every answer printed as it comes, to the
/// daemon's finish: `created`, with the request's id; a `delivered`
/// for each listener sent it, as each is; then `responded`, the answer
/// and who gave it — or the one line that refuses it. The command
/// waits for as long as nobody answers; ending it takes the request
/// back from everyone it was sent to.
pub async fn run(arguments: Arguments, target: &Target<'_>, emitter: &mut Emitter) -> Result<(), Error> {
    match arguments.below {
        Some(Below::RequestSchema(_)) => {
            return request_schema::<request::Frame>("diverge run requests create", emitter).await;
        }
        Some(Below::ResponseSchema(_)) => {
            return response_schema::<response::Frame>("diverge run requests create", emitter).await;
        }
        None => {}
    }
    let request: request::Frame = body(arguments.body.as_deref())?;
    let handle = target.dial().await?;
    shape::stream::<_, response::Frame, _>(&handle, &request, emitter, shape::never).await
}
