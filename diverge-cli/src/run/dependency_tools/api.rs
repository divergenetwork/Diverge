//! `diverge run dependency-tools api`: one HTTP request to a dependency tool's
//! API, its body this command's stdin, as
//! [`api`](diverge_sdk::daemon::endpoints::dependency_tools::api) states it.

use clap::Args;
use diverge_sdk::daemon::endpoints::dependency_tools::api::{client::execute, client::request, server::response};

use crate::run::shape::api::Said;
use crate::run::{Below, Emitter, Error, Target, body, request_schema, response_schema, shape};

/// One HTTP request to a dependency tool's API.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(subcommand_negates_reqs = true, args_conflicts_with_subcommands = true)]
pub struct Arguments {
    /// What the request carries, as a JSON object: see
    /// [`request::Frame`] — whose API, the `method`, the `path`
    /// beneath `/api`, and the `headers`; `relay` among its members
    /// has another daemon serve it.
    #[arg(value_name = "BODY")]
    pub body: Option<String>,
    /// What is beneath the request: `request-schema`, the JSON Schema
    /// of the body it takes, or `response-schema`, the JSON Schema of
    /// every answer — printed in place of putting any request.
    #[command(subcommand)]
    pub below: Option<Below>,
}

/// The request put, this command's stdin its body, the response's head
/// on stderr and its body on stdout (see
/// [`shape::api`](crate::run::shape::api)).
pub async fn run(arguments: Arguments, target: &Target<'_>, emitter: &mut Emitter) -> Result<(), Error> {
    match arguments.below {
        Some(Below::RequestSchema(_)) => {
            return request_schema::<request::Frame>("diverge run dependency-tools api", emitter).await;
        }
        Some(Below::ResponseSchema(_)) => {
            return response_schema::<response::Frame>("diverge run dependency-tools api", emitter).await;
        }
        None => {}
    }
    let request: request::Frame = body(arguments.body.as_deref())?;
    let handle = target.dial().await?;
    let (answers, api) = execute::execute(&handle, &request).await.map_err(shape::exchange::open)?;
    let writer = api.clone();
    let feed = move |bytes: Vec<u8>| {
        let api = writer.clone();
        async move { api.body(&bytes).await.is_ok() }
    };
    let end = move || async move {
        let _ = api.end().await;
    };
    shape::api::api(answers, feed, end, said).await
}

/// What an answer says.
fn said(frame: &response::Frame) -> Said {
    match frame {
        response::Frame::Head(_) => Said::Head,
        response::Frame::Body(bytes) => Said::Body(bytes.clone()),
        response::Frame::NotFound | response::Frame::Forbidden | response::Frame::NoApi | response::Frame::Error(_) => Said::Refused,
    }
}
