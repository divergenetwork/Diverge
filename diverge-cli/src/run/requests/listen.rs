//! `diverge run requests listen`: the requests made of this account
//! heard, and answered from standard input, as
//! [`listen`](diverge_sdk::daemon::endpoints::requests::listen) states
//! it.

use clap::Args;
use diverge_sdk::daemon::endpoints::requests::listen::client::channel_request::Respond;
use diverge_sdk::daemon::endpoints::requests::listen::client::{execute, request};
use diverge_sdk::daemon::endpoints::requests::listen::server::{channel_response, response};
use futures_util::StreamExt as _;
use futures_util::stream::FuturesUnordered;
use serde_json::json;
use tokio::io::{AsyncBufReadExt as _, BufReader};

use crate::run::shape::exchange;
use crate::run::{Below, Emitter, Error, Target, body, json as decode, request_schema, response_schema};

/// Hearing the requests made of this account, and answering them.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(subcommand_negates_reqs = true, args_conflicts_with_subcommands = true)]
pub struct Arguments {
    /// What the request carries, as a JSON object: see
    /// [`request::Frame`] — nothing but `relay`, which has another
    /// daemon serve it. Left out, `{}`.
    #[arg(value_name = "BODY")]
    pub body: Option<String>,
    /// What is beneath the request: `request-schema`, the JSON Schema
    /// of the body it takes, or `response-schema`, the JSON Schema of
    /// the lines the daemon's stream prints — printed in place of
    /// putting any request.
    #[command(subcommand)]
    pub below: Option<Below>,
}

/// The request put, and the listen held open. The rules:
///
/// 1. **What is heard is printed.** Each `request`, with its id and
///    its value, and each `withdrawn`, with its id, is a line as it
///    comes.
/// 2. **An answer is a line of standard input**: one JSON object,
///    `{"id": …, "response": …}`, the id a request came with and any
///    JSON value. An empty line is passed over. A line that is not
///    that object ends the command as an error.
/// 3. **Each answer's delivery is printed**, when the daemon says it:
///    `{"delivered": {"id": …}}`, the answer was taken, or
///    `{"not_delivered": {"id": …}}`, it was not. Answers are put as
///    they are read, none waiting on another, so deliveries are
///    printed in the order the daemon gives them.
/// 4. **Standard input ending ends nothing.** No more answers are
///    read, the ones put are still waited on, and the listen stays
///    open.
/// 5. **It ends with the daemon's finish** — a listen refused, as
///    `already_open` or `forbidden` — or with the command. It never
///    ends by itself.
pub async fn run(arguments: Arguments, target: &Target<'_>, emitter: &mut Emitter) -> Result<(), Error> {
    match arguments.below {
        Some(Below::RequestSchema(_)) => {
            return request_schema::<request::Frame>("diverge run requests listen", emitter).await;
        }
        Some(Below::ResponseSchema(_)) => {
            return response_schema::<response::Frame>("diverge run requests listen", emitter).await;
        }
        None => {}
    }
    let request: request::Frame = body(arguments.body.as_deref())?;
    let handle = target.dial().await?;
    let (mut heard, responder) = execute::execute(&handle, &request).await.map_err(exchange::open)?;
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    let mut reading = true;
    let mut answers = FuturesUnordered::new();
    loop {
        tokio::select! {
            // Rules 1 and 5.
            item = heard.next() => {
                let Some(item) = item else {
                    return Ok(());
                };
                emitter.emit(&item.map_err(exchange::stream)?).await?;
            }
            // Rules 2 and 4.
            line = lines.next_line(), if reading => match line {
                Ok(Some(line)) if line.trim().is_empty() => {}
                Ok(Some(line)) => {
                    let Respond { id, response } = decode(&line).map_err(Error::Arguments)?;
                    let responder = responder.clone();
                    answers.push(async move {
                        let delivery = responder.respond(id.clone(), response).await;
                        (id, delivery)
                    });
                }
                Ok(None) | Err(_) => reading = false,
            },
            // Rule 3.
            Some((id, delivery)) = answers.next(), if !answers.is_empty() => {
                let line = match delivery.map_err(|error| Error::Exchange(error.to_string()))? {
                    channel_response::Frame::Delivered => json!({ "delivered": { "id": id } }),
                    channel_response::Frame::NotDelivered => json!({ "not_delivered": { "id": id } }),
                };
                emitter.emit_value(line).await?;
            }
        }
    }
}
