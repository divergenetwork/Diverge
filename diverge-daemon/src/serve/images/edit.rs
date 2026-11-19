//! Changing the description of an image in the daemon's Store on a
//! provider.

use diverge_sdk::daemon::endpoints::images::edit::client::request;
use diverge_sdk::daemon::endpoints::images::edit::server::response::Frame;
use diverge_sdk::daemon::grant::images::Action;
use diverge_sdk::provider::endpoints::images::edit::client::execute::{ExecuteError, execute};
use diverge_sdk::provider::endpoints::images::edit::client::request::Frame as Ask;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;

use crate::daemon::Daemon;
use crate::judge::{self, Standing, Who};
use crate::serve::reply;
use crate::store;
use crate::volumes;

/// Answer the edit, and finish the scope.
pub async fn handle(scope: ScopeHandle, frame: request::Frame, who: Who, daemon: &Daemon) {
    let answer = match serve(&scope, frame, who, daemon).await {
        Ok(answer) => answer,
        Err(error) => Frame::Error(reply::failure(&error)),
    };
    reply::reply(&scope, &answer).await;
    scope.send_response_finish().await;
}

/// The edit's one answer. The rules:
///
/// 1. `Forbidden` with no `edit` grant over images at all.
/// 2. **The provider named**: `NotFound` for one the daemon keeps no
///    record of; `Forbidden` for one no `edit` grant reaches, by its
///    identity or its tags.
/// 3. `Error` for a provider not connected now.
/// 4. **Asked of the provider as the daemon**: its
///    [`images::edit`](diverge_sdk::provider::endpoints::images::edit),
///    the description as the request gave it, absent to clear it.
///    `Edited`; `NotFound` for a digest the daemon owns no image by
///    there; `Unsupported` for a provider with no Store.
async fn serve(scope: &ScopeHandle, frame: request::Frame, who: Who, daemon: &Daemon) -> Result<Frame, store::Error> {
    let standing = {
        let mut conn = daemon.store.acquire().await?;
        Standing::of(&mut conn, who).await?
    };
    // Rule 1.
    let Some(standing) = standing else {
        return Ok(Frame::Forbidden);
    };
    if !judge::images::holds(&standing, Action::Edit) {
        return Ok(Frame::Forbidden);
    }
    // Rule 2.
    let records = daemon.providers.all();
    let Some(record) = records.iter().find(|record| record.identity == frame.provider) else {
        return Ok(Frame::NotFound);
    };
    if !judge::images::over(&standing, Action::Edit, record) {
        return Ok(Frame::Forbidden);
    }
    scope.allowed().await;
    // Rule 3.
    let handle = match volumes::handle(daemon, &frame.provider).await {
        Ok(handle) => handle,
        Err(fail) => return Ok(Frame::Error(fail.error())),
    };
    // Rule 4.
    let ask = Ask {
        digest: frame.digest,
        description: frame.description,
    };
    Ok(match execute(&handle, &ask).await {
        Ok(()) => Frame::Edited,
        Err(ExecuteError::NotFound) => Frame::NotFound,
        Err(ExecuteError::Unsupported) => Frame::Unsupported,
        Err(ExecuteError::Provider(error)) => Frame::Error(error),
        Err(error) => Frame::Error(reply::failure(&error)),
    })
}
