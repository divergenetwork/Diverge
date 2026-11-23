//! Answering a merge, from a scope and a manager.

use diverge_sdk::shared::Identity;
use std::sync::Arc;

use crate::protocol::changes::Changes;
use crate::protocol::image_manager::ImageManager;
use crate::protocol::progress::Merging;
use diverge_sdk::provider::endpoints::images::merge::client::request;
use diverge_sdk::provider::endpoints::images::merge::server::response;
use diverge_sdk::provider::endpoints::info::server::response::Capabilities;
use diverge_sdk::shared::error::Error;
use diverge_sdk::wire::encode::{Encode, Writer};
use diverge_sdk::wire::server::scope_handle::ScopeHandle;

/// Make the image and end the scope.
///
/// The rules:
///
/// 1. A provider whose capabilities keep no Store answers
///    [`Unsupported`](response::Frame::Unsupported) and asks nothing.
/// 2. The manager makes one image of the platforms named — see
///    [`merge`](diverge_sdk::provider::endpoints::images::merge) — and
///    how it ended is the answer.
/// 3. Every listing open is told of the new image, by digest, when
///    one was made: a merge that made a digest the caller owned
///    already tells nothing.
/// 4. A manager that will not answer is its error, flattened.
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
    let frame = if capabilities.images.store.is_none() {
        response::Frame::Unsupported
    } else {
        match manager.merge(client_identity, &request.images).await {
            Ok(Merging::Merged(merged)) => {
                changes.changed(client_identity, &merged.digest);
                response::Frame::Merged(merged)
            }
            Ok(Merging::Exists(digest)) => response::Frame::Exists(digest),
            Ok(Merging::NotFound) => response::Frame::NotFound,
            Ok(Merging::PlatformNotHeld) => response::Frame::PlatformNotHeld,
            Ok(Merging::PlatformRepeated) => response::Frame::PlatformRepeated,
            Ok(Merging::InsufficientCapacity) => response::Frame::InsufficientCapacity,
            Err(error) => response::Frame::Error(error.into()),
        }
    };
    let mut buffer = Vec::new();
    if frame.encode(&mut Writer::new(&mut buffer)).is_ok() {
        scope.send_response(&buffer).await;
    }
    scope.send_response_finish().await;
}
