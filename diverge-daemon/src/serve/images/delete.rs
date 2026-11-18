//! Deleting an image in a Store of the daemon's, or some of its
//! platforms.

use diverge_sdk::daemon::endpoints::images::delete::client::request;
use diverge_sdk::daemon::endpoints::images::delete::server::response::Frame;
use diverge_sdk::daemon::grant::images::Action;
use diverge_sdk::provider::endpoints::images::delete::client::execute::{ExecuteError, execute};
use diverge_sdk::provider::endpoints::images::delete::client::request::Frame as Ask;
use diverge_sdk::provider::endpoints::images::refusal;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;

use crate::daemon::Daemon;
use crate::judge::{self, Standing, Who};
use crate::serve::reply;
use crate::store;
use crate::volumes;

/// Answer the delete and finish the scope.
pub async fn handle(scope: ScopeHandle, frame: request::Frame, who: Who, daemon: &Daemon) {
    let answer = match serve(&scope, frame, who, daemon).await {
        Ok(answer) => answer,
        Err(error) => Frame::Error(reply::failure(&error)),
    };
    reply::reply(&scope, &answer).await;
    scope.send_response_finish().await;
}

/// The rules:
///
/// 1. `Forbidden` with no `delete` grant over images at all.
/// 2. `NotFound` for a provider the daemon keeps no record of;
///    `Forbidden` for one no `delete` grant reaches, by its identity
///    or its tags.
/// 3. `Error` for a provider not connected now.
/// 4. **The provider is asked, as the daemon itself**, so what is
///    deleted is the daemon's own image there: its `images::delete`,
///    with the digest and the platforms as the request named them.
/// 5. **Its answers, relayed.** Deleted, in use, a platform not held
///    and unsupported are each the answer of the same name. A digest
///    the daemon owns no image by there, which the provider refuses
///    with [`refusal::unknown`], is `NotFound`; any other error of the
///    provider's is the `Error`, in its words; an exchange that broke
///    is the `Error`, in the daemon's.
/// 6. Nothing is done to the daemon's list: the provider tells the
///    listing the daemon keeps open on it, which the list follows.
async fn serve(scope: &ScopeHandle, frame: request::Frame, who: Who, daemon: &Daemon) -> Result<Frame, store::Error> {
    let standing = {
        let mut conn = daemon.store.acquire().await?;
        Standing::of(&mut conn, who).await?
    };
    // Rule 1.
    let Some(standing) = standing else {
        return Ok(Frame::Forbidden);
    };
    if !judge::images::holds(&standing, Action::Delete) {
        return Ok(Frame::Forbidden);
    }
    // Rule 2.
    let records = daemon.providers.all();
    let Some(record) = records.iter().find(|record| record.identity == frame.provider) else {
        return Ok(Frame::NotFound);
    };
    if !judge::images::over(&standing, Action::Delete, record) {
        return Ok(Frame::Forbidden);
    }
    scope.allowed().await;
    // Rule 3.
    let handle = match volumes::handle(daemon, &frame.provider).await {
        Ok(handle) => handle,
        Err(fail) => return Ok(Frame::Error(fail.error())),
    };
    // Rules 4 and 5.
    let ask = Ask {
        digest: frame.digest,
        platforms: frame.platforms,
    };
    Ok(match execute(&handle, &ask).await {
        Ok(()) => Frame::Deleted,
        Err(ExecuteError::InUse) => Frame::InUse,
        Err(ExecuteError::PlatformNotHeld) => Frame::PlatformNotHeld,
        Err(ExecuteError::Unsupported) => Frame::Unsupported,
        Err(ExecuteError::Provider(error)) if error == refusal::unknown(&ask.digest) => Frame::NotFound,
        Err(ExecuteError::Provider(error)) => Frame::Error(error),
        Err(error) => Frame::Error(reply::failure(&error)),
    })
}
