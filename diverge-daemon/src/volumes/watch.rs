//! Keeping a provider's mirror, for the connection's life.

use std::sync::Arc;

use diverge_sdk::provider::endpoints::volumes::list::client::{execute, request};
use diverge_sdk::provider::endpoints::volumes::list::server::response::Frame;
use diverge_sdk::shared::Identity;
use diverge_sdk::wire::client::handle::Handle;
use futures_util::StreamExt as _;

use super::Mirror;
use crate::daemon::{Daemon, Kind};

/// Open the provider's `volumes::list` on `handle` and keep the
/// provider's mirror — made and kept by the attach before this runs,
/// so a connection gone meanwhile takes it with its slot and none is
/// kept after — from it: every frame applied, and the word
/// that volumes changed said after each that changed something. A
/// listing that could not open, or that ends in the provider's error,
/// marks the mirror failed; the stream ending — the connection gone —
/// ends this, and the mirror goes with the connection's slot. The
/// listing is opened once per connection and never again, so the
/// provider's one-at-a-time refusal — see
/// [the endpoints](diverge_sdk::provider::endpoints#one-open-at-a-time)
/// — is never met; were it met, it marks the mirror failed as an error
/// does, and nothing is retried.
///
/// Once per connection, when the provider says its listing is whole,
/// the tags the daemon keeps on `provider`'s volumes are
/// [swept](sweep) against it.
pub async fn watch(daemon: Arc<Daemon>, provider: Identity, mirror: Arc<Mirror>, handle: Handle) {
    let (mut listing, _stop) = match execute::execute(&handle, &request::Frame).await {
        Ok(opened) => opened,
        Err(error) => {
            mirror.failed(error.to_string());
            daemon.live.changed(Kind::Volumes);
            return;
        }
    };
    while let Some(item) = listing.next().await {
        let changed = match item {
            Ok(frame) => {
                let listed = matches!(frame, Frame::Listed);
                let changed = mirror.apply(frame).await;
                // The first `listed` of the connection, and no later
                // one. Waited for, before the next frame is read: a
                // volume is tagged only once the mirror holds it, and
                // the mirror is fed here alone, so no volume listed
                // after these names can be tagged and then swept.
                if listed && changed {
                    let names: Vec<String> = mirror.volumes().await.into_iter().map(|volume| volume.name).collect();
                    sweep(&daemon, &provider, &names).await;
                }
                changed
            }
            Err(execute::ExecuteStreamError::Refused(error)) => {
                mirror.failed(super::describe(&error));
                true
            }
            Err(error) => {
                mirror.failed(error.to_string());
                true
            }
        };
        if changed {
            daemon.live.changed(Kind::Volumes);
        }
    }
}

/// The tags of every volume of `provider`'s that its whole listing
/// does not name, removed: `names` is that listing. The daemon keeps
/// nothing of a volume but its tags, and forgets them when it deletes
/// the volume; tags are left behind where it did not — the volume
/// deleted and the daemon ended before the tags went, or the volume
/// removed on the provider by another hand while the daemon was not
/// connected. A tag left behind is not idle: a volume later made
/// under the same name would bear it, and every grant that reaches by
/// that tag would reach the new volume. The rules:
///
/// - It runs once per connection, on the listing said whole, and the
///   listing is read on only once it is done. A listing still
///   arriving or one that failed sweeps nothing: only a whole listing
///   says what is not there.
/// - A volume the provider stops listing while connected is not swept
///   then: a volume the daemon deletes has its tags forgotten by the
///   delete, and any other is swept at the next connection.
/// - A provider that holds out no volumes is not listed, and its tags
///   are left.
/// - What is not in the listing is taken as gone for good. A provider
///   that comes back without a volume it will have again — a store
///   not mounted — has that volume's tags removed, and they are put
///   on again by hand.
/// - A store that refuses is let go: the tags stay, for the next
///   connection's sweep.
async fn sweep(daemon: &Daemon, provider: &Identity, names: &[String]) {
    let Ok(mut conn) = daemon.store.acquire().await else {
        return;
    };
    if let Ok(removed) = crate::store::volumes::keep_only(&mut conn, provider, names).await {
        if removed > 0 {
            daemon.live.changed(Kind::Volumes);
        }
    }
}
