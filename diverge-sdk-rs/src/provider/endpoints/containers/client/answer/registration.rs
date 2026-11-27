//! The container's registration, served by the caller: its
//! dependencies deployed, its permissions held.

use std::sync::Arc;

use super::send::{Stop, finish, respond};
use crate::provider::client::Registrar;
use crate::wire::client::handle::Handle;
use crate::shared::containers::dependencies::Template;
use crate::shared::containers::registration::response;
use crate::shared::permission::Grant;

/// One frame, deployed or the caller's refusal, then the finish.
pub(crate) async fn registration<T: Registrar>(
    handle: &Handle,
    scope: u32,
    channel: u32,
    id: String,
    dependencies: Vec<Template>,
    permissions: Vec<Grant>,
    registrar: Arc<T>,
) -> Result<(), Stop> {
    let frame = match registrar.register(id, dependencies, permissions).await {
        Ok(()) => response::Frame::Deployed,
        Err(error) => response::Frame::Error(error),
    };
    respond(handle, scope, channel, &frame).await?;
    finish(handle, scope, channel).await
}
