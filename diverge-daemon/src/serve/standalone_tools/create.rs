//! Creating a tool from a template.

use diverge_sdk::daemon::endpoints::standalone_tools::create::client::request;
use diverge_sdk::daemon::endpoints::standalone_tools::create::server::response::Frame;
use diverge_sdk::daemon::grant::standalone_tools::Make;
use std::sync::Arc;

use diverge_sdk::wire::server::scope_handle::ScopeHandle;

use crate::containers::{self, images};
use crate::daemon::{Daemon, Kind};
use crate::judge::{self, Standing, Who};
use crate::serve::inner::{self, Checked, Mounted, Secrets};
use crate::serve::reply;
use crate::store::{self, tools, tools_templates};

/// Answer the create and finish the scope.
pub async fn handle(scope: ScopeHandle, frame: request::Frame, who: Who, daemon: &Arc<Daemon>) {
    let answer = match serve(frame, who, daemon).await {
        Ok(answer) => answer,
        Err(error) => Frame::Error(reply::failure(&error)),
    };
    reply::reply(&scope, &answer).await;
    scope.send_response_finish().await;
}

/// `Forbidden` without the `create` grant, or for a vault key the
/// grants do not reach; the error for a template or a provider that is
/// not there; the `secrets` error for secrets that are not the
/// template's; `InUse` for a name a standalone or a connected tool
/// has, as [`tool_names`](crate::store::tool_names) states; else the
/// tool made, its index the next for its template,
/// nothing running: the container runs while an attached agent is
/// active.
///
/// The template's image is held to its document before anything is
/// made, and before the transaction that makes it begins, so nothing
/// of the store is held while a provider or a daemon is waited for:
///
/// 1. The grant and the template are read first, on a connection
///    given back at once; a caller without the grant, or a template
///    that is not there, is answered then, and nothing is asked.
/// 2. The image's document is found as [`images::document`] finds it —
///    the record, else whoever holds the image, the document then
///    left on record. An image with none is the `image` error.
/// 3. A document that is an agent's is the `image` error too: a tool
///    is made of a tool's image.
/// 4. Then the create, as stated above, its checks made again in its
///    own transaction.
async fn serve(frame: request::Frame, who: Who, daemon: &Arc<Daemon>) -> Result<Frame, store::Error> {
    // Rule 1.
    let image = {
        let mut conn = daemon.store.acquire().await?;
        let Some(standing) = Standing::of(&mut conn, who.clone()).await? else {
            return Ok(Frame::Forbidden);
        };
        if !judge::standalone_tools::make(&standing, Make::Create) {
            return Ok(Frame::Forbidden);
        }
        let Some(template) = tools_templates::by_id(&mut conn, &frame.inner.template, false).await? else {
            return Ok(Frame::Error(reply::failure(&"the tool template named is none the daemon has")));
        };
        containers::image(&template.template.image)
    };
    // Rules 2 and 3.
    let document = match images::document(daemon, &image).await? {
        Err(why) => return Ok(Frame::Error(reply::image(&why))),
        Ok(document) => {
            if document.agent().is_some() {
                return Ok(Frame::Error(reply::image("the image's document is an agent's, and a tool is made of a standalone tool's image")));
            }
            if document.dependency_tool().is_some() {
                return Ok(Frame::Error(reply::image(
                    "the image's document is a dependency tool's, which an agent's program declares for itself and no tool template makes",
                )));
            }
            document
        }
    };
    let mut tx = daemon.store.begin().await?;
    let Some(standing) = Standing::of(&mut tx, who).await? else {
        return Ok(Frame::Forbidden);
    };
    if !judge::standalone_tools::make(&standing, Make::Create) {
        return Ok(Frame::Forbidden);
    }
    let inner = frame.inner;
    let Some(template) = tools_templates::by_id(&mut tx, &inner.template, false).await? else {
        return Ok(Frame::Error(reply::failure(&"the tool template named is none the daemon has")));
    };
    // What the image needs and what the template adds is a limit.
    if let Err(why) = images::limits(&document, template.template.extra_memory, template.template.extra_disk) {
        return Ok(Frame::Error(reply::image(&why)));
    }
    match inner::secrets(&mut tx, &standing, document.secrets(), &inner.secrets).await? {
        Secrets::Ok => {}
        Secrets::Mismatch(what) => return Ok(Frame::Error(reply::secrets(&what))),
        Secrets::Missing(what) => return Ok(Frame::Error(reply::secrets(&what))),
        Secrets::Forbidden => return Ok(Frame::Forbidden),
    }
    match inner::providers(&mut tx, inner.provider.as_ref(), &inner.fuse_file_mounts, &inner.fuse_directory_mounts).await? {
        Checked::Ok(()) => {}
        Checked::Error(error) => return Ok(Frame::Error(reply::failure(&error))),
    }
    match inner::mounts(&mut tx, &standing, daemon, inner.provider.as_ref(), &inner.fuse_file_mounts, &inner.fuse_directory_mounts).await? {
        Mounted::Ok => {}
        Mounted::Forbidden => return Ok(Frame::Forbidden),
        Mounted::Error(error) => return Ok(Frame::Error(reply::failure(&error))),
        Mounted::Unsupported(what) => return Ok(Frame::Error(reply::unsupported(&what))),
    }
    let new = tools::New {
        template: inner.template,
        provider: inner.provider,
        name: frame.name,
        fuse_file_mounts: inner.fuse_file_mounts,
        fuse_directory_mounts: inner.fuse_directory_mounts,
        secrets: inner.secrets,
        creator: standing.creator.clone(),
    };
    match tools::create(&mut tx, &new).await? {
        tools::Created::Created(..) => {}
        tools::Created::Exists => return Ok(Frame::InUse),
    }
    tx.commit().await?;
    daemon.live.changed(Kind::StandaloneTools);
    daemon.live.changed(Kind::Volumes);
    daemon.live.changed(Kind::Vault);
    Ok(Frame::Created)
}
