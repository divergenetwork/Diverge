//! Finding the sources that can supply an image: providers, and remote
//! daemons, platform by platform.

use std::collections::HashSet;
use std::sync::Arc;

use diverge_sdk::daemon::endpoints::daemons::DaemonId;
use diverge_sdk::daemon::endpoints::images::read::client::{execute as daemon_execute, request as daemon_request};
use diverge_sdk::provider::endpoints::images::read::client::execute::More;
use diverge_sdk::provider::endpoints::images::read::client::{execute, request};
use diverge_sdk::shared::Identity;
use diverge_sdk::shared::containers::oci;
use diverge_sdk::shared::containers::request::Image;
use futures_util::future::BoxFuture;
use futures_util::stream::FuturesUnordered;
use futures_util::{FutureExt as _, StreamExt as _};
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;

use crate::daemon::Daemon;
use crate::daemons;
use crate::store;
use crate::store::daemons::Record;

/// Whom an ask may reach.
pub enum Reach {
    /// Every connected provider but `except`, and every remote daemon
    /// recorded: a provider asking on a run — which has looked in its
    /// own Store and its own registries already, and is `except` — or
    /// the daemon asking for itself, `except` then `None`. A provider
    /// asking needs no grant.
    Full { except: Option<Identity> },
    /// Exactly these providers and these remote daemons' records: what a
    /// command's grants reach.
    Granted { providers: HashSet<Identity>, daemons: Vec<Record> },
}

/// Which platforms of the image a search is for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Wanted {
    /// Any platform: the first source that can supply anything of the
    /// image ends the search.
    Any,
    /// Every one of these, each once: the search goes on until a
    /// source has been found for each.
    Every(Vec<String>),
}

/// Who answered held.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Holder {
    /// A connected provider, by its identity.
    Provider(Identity),
    /// A remote daemon, by the record it was reached through.
    Daemon(DaemonId),
}

/// One scope that answered held: a source an image's manifests, blobs
/// and proof are taken through. A provider's read scope and a remote
/// daemon's share their channels byte for byte, so either is held by
/// the same handle.
#[derive(Debug)]
pub struct Source {
    pub holder: Holder,
    pub scope: execute::ExecuteHandle,
}

/// Platforms a search found a source for.
#[derive(Debug, Clone)]
pub struct Found {
    /// The source that supplies them. A remote daemon that finds more
    /// later is the source of more than one of these.
    pub source: Arc<Source>,
    /// The platforms: wanted, and found by no earlier one of the
    /// search. Under [`Wanted::Any`], every platform the source said.
    pub platforms: Vec<String>,
}

/// A search for the sources of one image, running on a task of its
/// own: what [`search`] hands back.
///
/// The rules:
///
/// 1. **What is found comes as it is found.** [`next`](Self::next) is
///    the next [`Found`], and `None` once the search is settled: every
///    platform wanted found, every source answered, or
///    [`enough`](Self::enough) said.
/// 2. **A source handed out is the taker's.** The search cancels none
///    it has handed out; whoever took it lets it go.
/// 3. **Always stopped.** Whoever holds a search ends it with
///    [`stop`](Self::stop), settled or not: what was found and not
///    taken is cancelled there. No destructor does it.
pub struct Search {
    found: mpsc::UnboundedReceiver<Found>,
    enough: watch::Sender<bool>,
    /// Every source handed out, so that [`stop`](Self::stop) cancels
    /// none of them.
    taken: Vec<Arc<Source>>,
}

impl Search {
    /// Rule 1: what was found next, or `None` once settled.
    pub async fn next(&mut self) -> Option<Found> {
        let found = self.found.recv().await?;
        if !self.taken.iter().any(|taken| Arc::ptr_eq(taken, &found.source)) {
            self.taken.push(Arc::clone(&found.source));
        }
        Some(found)
    }

    /// Look no further: the search settles with what it has found, and
    /// lets go of every source it was still asking. Waits for nothing.
    pub fn enough(&self) {
        self.enough.send_replace(true);
    }

    /// Rule 3: the search ended, and every source found and not taken
    /// cancelled, each on a task of its own.
    pub async fn stop(mut self) {
        self.enough();
        self.found.close();
        while let Some(found) = self.found.recv().await {
            if !self.taken.iter().any(|taken| Arc::ptr_eq(taken, &found.source)) {
                self.taken.push(Arc::clone(&found.source));
                let_go(found.source);
            }
        }
    }
}

/// Search for the sources of `image`: every source `reach` reaches
/// asked at once, and what each adds handed back as it answers. It
/// returns at once; the asking is a task of its own.
///
/// The rules:
///
/// 1. **Who is asked.** Every connected provider `reach` reaches whose
///    capabilities keep a Store or list a registry, the only providers
///    that can hold anything; and every remote daemon's record `reach`
///    reaches whose remote is neither in `ignore` nor this daemon
///    itself — the identity it is.
/// 2. **Asked alike, all at once.** Each provider is asked by opening an
///    `images::read` scope with the image, references and all; each
///    remote daemon by opening its own `images::read` scope with the
///    image, the platforms wanted and `ignore`, this daemon's identity
///    added, on the connection [`daemons::connect`] holds or opens for
///    the record — the routes found as for every relay. A record that
///    will not connect is a no.
/// 3. **What a source adds.** A source that answers held says which
///    platforms it can supply. Those wanted that no source before it
///    supplied are one [`Found`], and are wanted no more. A remote
///    daemon says more as it finds more, each a `Found` of its own
///    where it adds anything.
/// 4. **A source that adds nothing is let go.** A provider at once; a
///    remote daemon, which may yet find more, once the search settles.
/// 5. **Settled.** The search ends when every platform wanted has been
///    found — under [`Wanted::Any`], when one source has — when every
///    source has answered and every remote daemon kept has settled,
///    when [`Search::enough`] is said, or when nobody holds the search
///    any more.
/// 6. **Nothing is waited for at the end.** Every ask still open goes
///    on in a task of its own, cancelled as it answers; every remote
///    daemon kept that is still looking is told `enough`, so that it
///    lets go of what it asks in turn; and every one of these is sent
///    without its answer being awaited.
/// 7. **Everything else is no.** A not found, an unsupported, a
///    forbidden, an error and a scope that would not open are all no.
pub fn search(daemon: &Arc<Daemon>, image: &Image, reach: Reach, ignore: &[Identity], wanted: Wanted) -> Search {
    let (found, receiver) = mpsc::unbounded_channel();
    let (enough, stopped) = watch::channel(false);
    tokio::spawn(run(Arc::clone(daemon), image.clone(), reach, ignore.to_vec(), wanted, found, stopped));
    Search {
        found: receiver,
        enough,
        taken: Vec::new(),
    }
}

/// The first source [`search`] finds that supplies anything wanted,
/// its scope held, or `None`; the search is stopped behind it. Under
/// [`Wanted::Every`], a source of any one of the platforms.
pub async fn locate(daemon: &Arc<Daemon>, image: &Image, reach: Reach, ignore: &[Identity], wanted: Wanted) -> Option<Arc<Source>> {
    let mut search = search(daemon, image, reach, ignore, wanted);
    let found = search.next().await;
    search.stop().await;
    found.map(|found| found.source)
}

/// A provider asked whether its caller, the daemon, holds `image`:
/// the name the daemon answers with where a source was found and is
/// kept in `pulls`, and `None` where there is none. What a run's
/// answerer answers a `has` with.
///
/// The rules:
///
/// 1. **A platform the provider runs.** What is wanted is any one of
///    the platforms `asking` says it runs containers for; a provider
///    that says none wants any platform. A source that holds the image
///    for other platforms alone is not the run's.
/// 2. **Found as [`locate`] finds one**, over the full reach: every
///    connected provider but `asking` — which has looked in its own
///    Store and its own registries already — and every remote daemon
///    recorded. A provider asking needs no grant.
/// 3. **Kept.** The source found is kept in `pulls`, through which
///    the image's manifests, blobs and proof are then taken; a `pulls`
///    released already keeps none, and the answer is `None`.
/// 4. **Named.** The answer is [`name`](super::name) of the digest.
pub async fn held(daemon: &Arc<Daemon>, pulls: &super::Pulls, image: &Image, asking: &Identity) -> Option<String> {
    // Rule 1.
    let runs = daemon.live.capabilities(asking).await.and_then(|info| info.capabilities.containers.as_ref().map(|containers| containers.platforms.clone()));
    let wanted = match runs {
        Some(platforms) if !platforms.is_empty() => Wanted::Every(platforms),
        _ => Wanted::Any,
    };
    let reach = Reach::Full { except: Some(asking.clone()) };
    let source = locate(daemon, image, reach, &[], wanted).await?;
    pulls.keep(source).await.then(|| super::name(&image.digest))
}

/// A remote daemon that answered held, while the search goes on.
struct Remote {
    source: Arc<Source>,
    /// Whether it supplied anything: rule 4 of [`search`].
    kept: bool,
    /// Whether it said it looks no further, or ended.
    settled: bool,
    /// The task reading what more it says.
    watcher: JoinHandle<()>,
}

/// The search itself: rules 3 to 6 of [`search`]. It owns what is
/// still wanted, and nothing else writes it; every ask and every
/// remote daemon's later word reaches it as a message.
async fn run(daemon: Arc<Daemon>, image: Image, reach: Reach, ignore: Vec<Identity>, wanted: Wanted, found: mpsc::UnboundedSender<Found>, mut stopped: watch::Receiver<bool>) {
    let mut asking = asks(&daemon, &image, &reach, &ignore, &wanted).await;
    let mut left = wanted;
    let mut any = false;
    let (said, mut saying) = mpsc::unbounded_channel::<(usize, More)>();
    let mut remotes: Vec<Remote> = Vec::new();
    let mut looking = 0usize;
    loop {
        // Rule 5.
        let complete = match &left {
            Wanted::Any => any,
            Wanted::Every(platforms) => platforms.is_empty(),
        };
        if complete || (asking.is_empty() && looking == 0) {
            break;
        }
        tokio::select! {
            Some(answered) = asking.next(), if !asking.is_empty() => {
                let Some(source) = answered else {
                    continue;
                };
                let source = Arc::new(source);
                // Rule 3.
                let added = take(&mut left, source.scope.platforms());
                let kept = added.is_some();
                if let Some(platforms) = added {
                    any = true;
                    if found.send(Found { source: Arc::clone(&source), platforms }).is_err() {
                        let_go(source);
                        break;
                    }
                }
                if matches!(source.holder, Holder::Daemon(_)) {
                    let index = remotes.len();
                    let watcher = tokio::spawn(watch_remote(Arc::clone(&source), index, said.clone()));
                    remotes.push(Remote { source, kept, settled: false, watcher });
                    looking += 1;
                } else if !kept {
                    // Rule 4.
                    let_go(source);
                }
            }
            Some((index, more)) = saying.recv(), if looking > 0 => match more {
                More::Held(platforms) => {
                    if let Some(platforms) = take(&mut left, &platforms) {
                        any = true;
                        remotes[index].kept = true;
                        if found.send(Found { source: Arc::clone(&remotes[index].source), platforms }).is_err() {
                            break;
                        }
                    }
                }
                More::Settled | More::End(_) => {
                    remotes[index].settled = true;
                    looking -= 1;
                }
            },
            _ = stopped.changed() => break,
            () = found.closed() => break,
        }
    }
    // Rule 6.
    cancel_rest(asking);
    for remote in remotes {
        remote.watcher.abort();
        if !remote.kept {
            let_go(remote.source);
        } else if !remote.settled {
            tokio::spawn(async move {
                let _ = remote.source.scope.enough().await;
            });
        }
    }
}

/// Rule 3 of [`search`]: the platforms of `said` that are still
/// wanted, taken out of `left`, or `None` where there is none. Under
/// [`Wanted::Any`], every platform said, whatever it is and even where
/// the source said none: a source that holds the image is one.
fn take(left: &mut Wanted, said: &[String]) -> Option<Vec<String>> {
    match left {
        Wanted::Any => Some(said.to_vec()),
        Wanted::Every(platforms) => {
            let mut added: Vec<String> = Vec::new();
            platforms.retain(|platform| {
                let adds = said.contains(platform);
                if adds {
                    added.push(platform.clone());
                }
                !adds
            });
            (!added.is_empty()).then_some(added)
        }
    }
}

/// What a remote daemon says after its first held, each sent to the
/// search as it comes, to its settled or its end.
async fn watch_remote(source: Arc<Source>, index: usize, said: mpsc::UnboundedSender<(usize, More)>) {
    loop {
        let more = source.scope.more().await;
        let last = !matches!(more, More::Held(_));
        if said.send((index, more)).is_err() || last {
            break;
        }
    }
}

/// `source` cancelled, on a task of its own: nothing waits for it.
pub fn let_go(source: Arc<Source>) {
    tokio::spawn(async move {
        let _ = source.scope.cancel().await;
    });
}

/// Every ask of who can supply an image, each answering its source,
/// held, or `None`: what [`asks`] makes, taken as each answers.
type Asks = FuturesUnordered<BoxFuture<'static, Option<Source>>>;

/// Rules 1 and 2 of [`search`]: every ask, made and not yet awaited,
/// so that all of them run at once as the set is taken from. Each
/// answers its source when its scope answered held, and `None` for
/// everything else, rule 7.
async fn asks(daemon: &Arc<Daemon>, image: &Image, reach: &Reach, ignore: &[Identity], wanted: &Wanted) -> Asks {
    // Rule 1.
    let own = daemon.authenticator.own().cloned();
    let asking: Asks = FuturesUnordered::new();
    let frame = Arc::new(request::Frame(oci::has::request::Request { image: image.clone() }));
    for identity in daemon.live.connected_providers().await {
        let reached = match reach {
            Reach::Full { except } => except.as_ref() != Some(&identity),
            Reach::Granted { providers, .. } => providers.contains(&identity),
        };
        if !reached {
            continue;
        }
        let supplies = daemon
            .live
            .capabilities(&identity)
            .await
            .is_some_and(|info| info.capabilities.images.store.is_some() || !info.capabilities.images.registries.is_empty());
        if !supplies {
            continue;
        }
        let Some(handle) = daemon.live.provider(&identity).await else {
            continue;
        };
        let frame = Arc::clone(&frame);
        // Rule 2.
        asking.push(
            async move {
                let scope = execute::execute(&handle, &frame).await.ok()?;
                Some(Source { holder: Holder::Provider(identity), scope })
            }
            .boxed(),
        );
    }
    let records = match reach {
        Reach::Full { .. } => recorded(daemon).await,
        Reach::Granted { daemons, .. } => daemons.clone(),
    };
    let mut passed: Vec<Identity> = ignore.to_vec();
    passed.extend(own.clone());
    let passed = Arc::new(passed);
    let platforms = match wanted {
        Wanted::Any => None,
        Wanted::Every(platforms) => Some(platforms.clone()),
    };
    for record in records {
        if ignore.contains(&record.daemon) || own.as_ref() == Some(&record.daemon) {
            continue;
        }
        let daemon = Arc::clone(daemon);
        let image = image.clone();
        let passed = Arc::clone(&passed);
        let platforms = platforms.clone();
        // Rule 2.
        asking.push(
            async move {
                let peer = daemons::connect(&daemon, &record).await.ok()?;
                let frame = daemon_request::Frame { image, platforms, ignore: passed.to_vec(), relay: None };
                let scope = daemon_execute::execute(&peer.connected.handle, &frame).await.ok()?;
                Some(Source { holder: Holder::Daemon(record.id), scope })
            }
            .boxed(),
        );
    }
    asking
}

/// Rule 6 of [`search`], for the asks left once it is settled: they
/// go on in a task of their own, and each that answers held is
/// cancelled as it answers, so no scope is left held with nobody to
/// take from it.
fn cancel_rest(mut asking: Asks) {
    if asking.is_empty() {
        return;
    }
    tokio::spawn(async move {
        while let Some(answered) = asking.next().await {
            if let Some(late) = answered {
                let _ = late.scope.cancel().await;
            }
        }
    });
}

/// Every remote daemon's record: none where the store cannot be read,
/// so that a failing store leaves the providers to answer.
async fn recorded(daemon: &Arc<Daemon>) -> Vec<Record> {
    let Ok(mut conn) = daemon.store.acquire().await else {
        return Vec::new();
    };
    store::daemons::all(&mut conn).await.unwrap_or_default()
}
