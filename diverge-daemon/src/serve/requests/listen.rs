//! Hearing the requests made of the caller, and answering them.

use std::sync::Arc;

use diverge_sdk::daemon::endpoints::requests::listen::client::{channel_request, request};
use diverge_sdk::daemon::endpoints::requests::listen::server::channel_response::Frame as Delivery;
use diverge_sdk::daemon::endpoints::requests::listen::server::response::{Asked, Frame, Withdrawn};
use diverge_sdk::wire::decode::Decode as _;
use diverge_sdk::wire::encode::{Encode as _, Writer};
use diverge_sdk::wire::frame::client::ClientFrame;
use diverge_sdk::wire::server::scope_handle::ScopeHandle;
use tokio::sync::broadcast::error::RecvError;

use super::{catch_up, named};
use crate::daemon::{Daemon, Kind};
use crate::judge::{Standing, Who};
use crate::requests::{Party, Told};
use crate::serve::reply;
use crate::store;

/// Serve the listen to its end, and finish the scope.
pub async fn handle(scope: ScopeHandle, _frame: request::Frame, who: Who, daemon: &Arc<Daemon>) {
    if let Err(error) = serve(&scope, who, daemon).await {
        reply::reply(&scope, &Frame::Error(reply::failure(&error))).await;
    }
    scope.send_response_finish().await;
}

/// The rules:
///
/// 1. `Forbidden` for an account that is none there is, and for a tool
///    whose row is gone: nobody is there to listen as.
/// 2. **One listen for each party.** The party's one slot is claimed,
///    or the answer is `AlreadyOpen` and nothing changed: see
///    [`Party`], and [`Requests::listen`](crate::requests::Requests::listen).
///    No grant is asked: a listener hears what was made of it.
/// 3. **Caught up, then kept up.** Every request pending now and made
///    of the party is delivered — [`catch_up`] — and each later one
///    reaches the party's queue from its creator. The look is made
///    again whenever accounts, roles or tools change, since a tag or a
///    grant changed then may be what makes a pending request the
///    party's.
/// 4. **Told in order.** Each word on the party's queue is sent as it
///    comes: a `Request`, or a `Withdrawn`.
/// 5. **An answer, on its channel.** A `Respond` is put to the hub,
///    which takes it or does not — one answer for a request, whoever
///    else answers at the same moment — and the channel is answered
///    `Delivered` or `NotDelivered`, then finished. A channel request
///    that does not decode is finished with nothing.
/// 6. **The end.** The caller's `Cancel`, the caller gone, or the
///    caller reading no more ends it; the slot is released, whatever
///    ended it.
async fn serve(scope: &ScopeHandle, who: Who, daemon: &Arc<Daemon>) -> Result<(), store::Error> {
    let party = Party::of(&who);
    // Rule 1.
    let name = {
        let mut conn = daemon.store.acquire().await?;
        let Some(standing) = Standing::of(&mut conn, who).await? else {
            reply::reply(scope, &Frame::Forbidden).await;
            return Ok(());
        };
        named(&mut conn, &party, &standing).await?
    };
    let Some(name) = name else {
        reply::reply(scope, &Frame::Forbidden).await;
        return Ok(());
    };
    // Rule 2.
    let Some(mut told) = daemon.live.requests.listen(party.clone(), name).await else {
        reply::reply(scope, &Frame::AlreadyOpen).await;
        return Ok(());
    };
    scope.allowed().await;
    let served = listening(scope, daemon, &party, &mut told).await;
    // Rule 6.
    daemon.live.requests.unlisten(&party).await;
    served
}

/// Rules 3 to 6 of [`serve`], between the slot claimed and the slot
/// released.
async fn listening(scope: &ScopeHandle, daemon: &Arc<Daemon>, party: &Party, told: &mut tokio::sync::mpsc::UnboundedReceiver<Told>) -> Result<(), store::Error> {
    // Subscribed before the first look, so a change between the two is
    // a word waiting and not one missed.
    let mut words = daemon.live.changes();
    // Rule 3.
    catch_up(daemon, party).await?;
    loop {
        tokio::select! {
            // Rule 4.
            word = told.recv() => {
                let frame = match word {
                    Some(Told::Request { id, request, from }) => Frame::Request(Asked { id, request: (*request).clone(), from }),
                    Some(Told::Withdrawn(id)) => Frame::Withdrawn(Withdrawn { id }),
                    None => return Ok(()),
                };
                reply::reply(scope, &frame).await;
            }
            // Rule 3.
            word = words.recv() => match word {
                Ok(Kind::Accounts | Kind::Roles | Kind::StandaloneTools | Kind::DependencyTools) | Err(RecvError::Lagged(_)) => catch_up(daemon, party).await?,
                Ok(_) => {}
                Err(RecvError::Closed) => return Ok(()),
            },
            // Rules 5 and 6.
            request = scope.recv_channel_request() => {
                let Some(bytes) = request else {
                    return Ok(());
                };
                let Ok(ClientFrame::ChannelRequest { channel, payload, .. }) = ClientFrame::decode(&bytes) else {
                    continue;
                };
                match channel_request::Frame::decode(payload) {
                    Ok(channel_request::Frame::Cancel) => return Ok(()),
                    Ok(channel_request::Frame::Respond(respond)) => {
                        let delivery = match daemon.live.requests.respond(&respond.id, party, respond.response).await {
                            true => Delivery::Delivered,
                            false => Delivery::NotDelivered,
                        };
                        let mut bytes = Vec::new();
                        if delivery.encode(&mut Writer::new(&mut bytes)).is_ok() {
                            scope.send_channel_response(channel, &bytes).await;
                        }
                        scope.send_channel_response_finish(channel).await;
                    }
                    Err(_) => scope.send_channel_response_finish(channel).await,
                }
            }
            () = scope.responses_gone() => return Ok(()),
        }
    }
}
