//! Answering a publish, from a scope and a manager.

use diverge_sdk::shared::Identity;
use std::sync::Arc;

use futures_util::StreamExt as _;
use serde_json::json;

use crate::protocol::changes::Changes;
use crate::protocol::image_manager::ImageManager;
use crate::protocol::progress::Publishing;
use diverge_sdk::provider::endpoints::images::publish::client::request;
use diverge_sdk::provider::endpoints::images::publish::server::response;
use diverge_sdk::provider::endpoints::info::server::response::Capabilities;
use diverge_sdk::shared::error::Error;
use diverge_sdk::wire::encode::{Encode, Writer};
use diverge_sdk::wire::server::scope_handle::ScopeHandle;

/// Send the image to the registry, saying what is uploaded as it goes,
/// and end the scope.
///
/// The rules:
///
/// 1. A provider whose capabilities keep no Store answers
///    [`Unsupported`](response::Frame::Unsupported) and asks nothing.
/// 2. The manager publishes — see
///    [`publish`](diverge_sdk::provider::endpoints::images::publish) —
///    and every item it yields is sent as the endpoint's frame, a
///    blob uploaded as it lands, until the one that ends it.
/// 3. Every listing open is told of the image, by digest, when it is
///    published: its references may have gained one.
/// 4. A caller that stops reading ends the publish where it is: the
///    stream is dropped, and what the registry already took it keeps.
/// 5. A publish whose stream ended without saying how is an error,
///    and nothing is known to be published.
/// 6. The finish.
///
/// The publish is not one of the endpoints held one at a time per
/// connection: two on one connection run beside each other.
pub async fn handle<M>(
    scope: ScopeHandle,
    request: request::Frame,
    client_identity: &Identity,
    manager: &M,
    changes: Arc<Changes>,
    capabilities: &Capabilities,
) where
    M: ImageManager,
    M::Error: Into<Error>,
{
    if capabilities.images.store.is_none() {
        send(&scope, &response::Frame::Unsupported).await;
        scope.send_response_finish().await;
        return;
    }
    let publish = manager
        .publish(client_identity, &request.digest, &request.reference, &request.tags, request.credential.as_ref())
        .await;
    let mut publish = Box::pin(publish);
    let mut ended = false;
    loop {
        let item = tokio::select! {
            item = publish.next() => item,
            () = scope.responses_gone() => {
                ended = true;
                break;
            }
        };
        let Some(item) = item else {
            break;
        };
        let frame = match item {
            Ok(Publishing::Blob(blob)) => response::Frame::Blob(blob),
            Ok(Publishing::Published(published)) => {
                changes.changed(client_identity, &published.digest);
                ended = true;
                response::Frame::Published(published)
            }
            Ok(end) => {
                ended = true;
                match end {
                    Publishing::NotFound => response::Frame::NotFound,
                    Publishing::Incomplete => response::Frame::Incomplete,
                    Publishing::RegistryUnlisted => response::Frame::RegistryUnlisted,
                    Publishing::Malformed => response::Frame::Malformed,
                    Publishing::Unauthorized => response::Frame::Unauthorized,
                    Publishing::Denied => response::Frame::Denied,
                    // The two that carry something are matched above.
                    Publishing::Blob(_) | Publishing::Published(_) => response::Frame::Error(unfinished()),
                }
            }
            Err(error) => {
                ended = true;
                response::Frame::Error(error.into())
            }
        };
        send(&scope, &frame).await;
        if ended {
            break;
        }
    }
    drop(publish);
    if !ended {
        send(&scope, &response::Frame::Error(unfinished())).await;
    }
    scope.send_response_finish().await;
}

/// The error for a publish whose stream ended without the image
/// published, a refusal, or an error.
///
/// ```json
/// {"kind":"publish","error":"the publish ended without a result"}
/// ```
fn unfinished() -> Error {
    Error(json!({
        "kind": "publish",
        "error": "the publish ended without a result",
    }))
}

/// One frame sent as a response.
async fn send(scope: &ScopeHandle, frame: &response::Frame) {
    let mut buffer = Vec::new();
    if frame.encode(&mut Writer::new(&mut buffer)).is_ok() {
        scope.send_response(&buffer).await;
    }
}
