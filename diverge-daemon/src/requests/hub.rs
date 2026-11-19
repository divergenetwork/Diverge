//! Every pending request and every listener, behind one lock.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use diverge_sdk::daemon::creator::Creator;
use diverge_sdk::daemon::endpoints::requests::RequestId;
use diverge_sdk::daemon::endpoints::requests::create::client::request::To;
use serde_json::Value;
use tokio::sync::{Mutex, mpsc};

use super::Party;
use crate::judge::Standing;

/// What a listener is told, in the order it is to be sent.
#[derive(Debug, Clone)]
pub enum Told {
    /// A request made of it.
    Request { id: RequestId, request: Arc<Value> },
    /// A request it was sent is wanted no more.
    Withdrawn(RequestId),
}

/// What becomes of a request, as its creator is told it, in order.
#[derive(Debug, Clone)]
pub enum Became {
    /// A listener was sent it: who.
    Delivered(Creator),
    /// It was answered, and is over: the answer, and who gave it.
    Responded { response: Value, from: Creator },
}

/// A pending request, as whoever judges a listener against it needs
/// it: what it is made of, and by whose grants.
#[derive(Debug, Clone)]
pub struct Candidate {
    /// The request.
    pub id: RequestId,
    /// Whom its creator made it of.
    pub to: Arc<To>,
    /// Its creator's grants, as they were at its create.
    pub standing: Arc<Standing>,
}

/// One pending request.
struct Pending {
    /// Its value, as every listener is sent it.
    request: Arc<Value>,
    /// Whom it is made of.
    to: Arc<To>,
    /// Its creator's grants at its create.
    standing: Arc<Standing>,
    /// Every listener it has been sent to: never sent twice, and the
    /// ones its end is told.
    delivered: HashSet<Party>,
    /// The way back to its creator's scope.
    creator: mpsc::UnboundedSender<Became>,
}

/// One open listen.
struct Listener {
    /// How its party is named to a creator.
    named: Creator,
    /// The way to its scope.
    told: mpsc::UnboundedSender<Told>,
}

/// What the lock protects.
#[derive(Default)]
struct State {
    pending: HashMap<RequestId, Pending>,
    listeners: HashMap<Party, Listener>,
}

/// Every pending request and every open listen: all the state requests
/// have. The rules:
///
/// 1. **One lock, never held across a wait.** Every method takes the
///    lock, changes maps and puts words on unbounded queues, and lets
///    it go: nothing in it awaits the store, the wire or another
///    party. So no caller is ever blocked by a slow listener or a
///    slow creator.
/// 2. **A request is pending until it is taken out, once.**
///    [`respond`](Self::respond) and [`withdraw`](Self::withdraw) each
///    take it out of the map under the lock, and whoever takes it out
///    is the one that ended it. Two answers at once are two calls,
///    one after the other: the first finds the request and takes it,
///    the second finds none.
/// 3. **Sent once to a party.** [`deliver`](Self::deliver) puts a
///    party in the request's `delivered` set and sends only where it
///    was not there: however many times a party is found eligible —
///    by the create, by its own listen opening, by a later look — it
///    is sent the request once, and the creator is told once.
/// 4. **A party's words are in order.** A listener's request is
///    queued by `deliver` and its withdrawal by `respond` or
///    `withdraw`, each under the lock, so a withdrawal never precedes
///    the request it withdraws.
/// 5. **Judged outside.** Whether a party is one a request is made of
///    is asked of the store, so it is not asked here: a caller takes
///    the [`Candidate`]s or the parties under the lock, judges them
///    with the lock let go, and calls `deliver` for those that pass.
///    Rule 3 is what makes that safe to do from two sides at once.
/// 6. **A listener gone keeps nothing and loses nothing.** Its slot is
///    released; what it was sent stays pending, and it stays in each
///    request's `delivered`, so listening again is not sent the same
///    request twice — and is not sent it again at all. It is sent the
///    withdrawals of nothing it was not, since a queue that is gone
///    takes no word.
#[derive(Default)]
pub struct Requests {
    state: Mutex<State>,
}

impl Requests {
    /// Nothing pending, nobody listening.
    pub fn new() -> Self {
        Requests::default()
    }

    /// `party` begins to listen, named `named` to creators: the queue
    /// its scope is told through, or `None` where it has a listen open
    /// already and nothing changed.
    pub async fn listen(&self, party: Party, named: Creator) -> Option<mpsc::UnboundedReceiver<Told>> {
        let mut state = self.state.lock().await;
        if state.listeners.contains_key(&party) {
            return None;
        }
        let (told, receiver) = mpsc::unbounded_channel();
        state.listeners.insert(party, Listener { named, told });
        Some(receiver)
    }

    /// `party` listens no more: rule 6.
    pub async fn unlisten(&self, party: &Party) {
        self.state.lock().await.listeners.remove(party);
    }

    /// Every pending request `party` has not been sent: what a
    /// listener is judged against, when it opens and when its record
    /// changes.
    pub async fn candidates(&self, party: &Party) -> Vec<Candidate> {
        let state = self.state.lock().await;
        state
            .pending
            .iter()
            .filter(|(_, pending)| !pending.delivered.contains(party))
            .map(|(id, pending)| Candidate {
                id: id.clone(),
                to: Arc::clone(&pending.to),
                standing: Arc::clone(&pending.standing),
            })
            .collect()
    }

    /// A request begins to be pending: the queue its creator's scope
    /// is told through, and every party listening now, for the creator
    /// to judge. The request is in the map before the parties are
    /// read, in the one critical section, so a listener that opens a
    /// moment after is one that finds it among its
    /// [`candidates`](Self::candidates).
    pub async fn create(&self, id: RequestId, request: Value, to: To, standing: Standing) -> (mpsc::UnboundedReceiver<Became>, Vec<Party>) {
        let (creator, receiver) = mpsc::unbounded_channel();
        let mut state = self.state.lock().await;
        state.pending.insert(
            id,
            Pending {
                request: Arc::new(request),
                to: Arc::new(to),
                standing: Arc::new(standing),
                delivered: HashSet::new(),
                creator,
            },
        );
        (receiver, state.listeners.keys().cloned().collect())
    }

    /// Rule 3: `party` sent the request `id`, where the request is
    /// pending, the party is listening, and it was not sent it before.
    /// The listener is told the request and the creator the delivery.
    pub async fn deliver(&self, id: &RequestId, party: &Party) {
        let mut state = self.state.lock().await;
        let State { pending, listeners } = &mut *state;
        let (Some(pending), Some(listener)) = (pending.get_mut(id), listeners.get(party)) else {
            return;
        };
        if !pending.delivered.insert(party.clone()) {
            return;
        }
        let _ = listener.told.send(Told::Request {
            id: id.clone(),
            request: Arc::clone(&pending.request),
        });
        let _ = pending.creator.send(Became::Delivered(listener.named.clone()));
    }

    /// Rule 2: `party`'s answer to the request `id`: whether it was
    /// taken. It is taken where the request is pending and was sent to
    /// this party, and taking it ends the request: the creator is told
    /// the answer and who gave it, and every OTHER party it was sent
    /// to is told it is withdrawn. The party that answered is told
    /// nothing here: its `true` is that word.
    pub async fn respond(&self, id: &RequestId, party: &Party, response: Value) -> bool {
        let mut state = self.state.lock().await;
        let sent = state.pending.get(id).is_some_and(|pending| pending.delivered.contains(party));
        let Some(named) = state.listeners.get(party).map(|listener| listener.named.clone()) else {
            return false;
        };
        if !sent {
            return false;
        }
        let Some(pending) = state.pending.remove(id) else {
            return false;
        };
        let _ = pending.creator.send(Became::Responded { response, from: named });
        for other in pending.delivered.iter().filter(|other| *other != party) {
            if let Some(listener) = state.listeners.get(other) {
                let _ = listener.told.send(Told::Withdrawn(id.clone()));
            }
        }
        true
    }

    /// Rule 2: the request `id` taken back by its creator, or with its
    /// creator's scope: every party it was sent to is told it is
    /// withdrawn. One already answered, or taken back, is not there,
    /// and nothing is told.
    pub async fn withdraw(&self, id: &RequestId) {
        let mut state = self.state.lock().await;
        let Some(pending) = state.pending.remove(id) else {
            return;
        };
        for party in &pending.delivered {
            if let Some(listener) = state.listeners.get(party) {
                let _ = listener.told.send(Told::Withdrawn(id.clone()));
            }
        }
    }
}
