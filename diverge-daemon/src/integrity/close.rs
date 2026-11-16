//! A run a log still holds deployed, closed.

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use diverge_sdk::daemon::lifecycle::{Provider, Reason, Undeployed};
use diverge_sdk::shared::Identity;
use serde_json::Value;

use super::Error;
use crate::containers::lifecycle::{Ask, RunBook};
use crate::logs::{self, Tail};

/// A run that was closed here: where it was deployed, and whether its
/// log held it working.
#[derive(Debug)]
pub struct Closed {
    /// The provider its `deployed` named.
    pub provider: Identity,
    /// Whether an `active` stood unanswered.
    pub active: bool,
}

/// `book`'s log under `logs` closed, if it stands deployed; `None`
/// where it does not, which is a log that ends `undeployed`, one with
/// no `deployed` in it, and one never appended to.
///
/// The daemon holds where a log stands in memory, and a daemon just
/// started holds every log undeployed without reading it: so a run
/// whose daemon died is one no later run would ever close. Where the
/// log stands is read here from the log, backwards from its end, by
/// each item's `type`:
///
/// - `undeployed` met first: the last run was closed. Nothing is
///   done.
/// - `deployed` met first: the run it opened was never closed, and
///   its `identity` is the provider. The walk ends there, since an
///   ask is kept only while a run is deployed.
/// - `active` or `inactive`, the first of either met: whether the run
///   was working.
/// - `call` or `read`: an ask. It is open unless an answer met before
///   it — later in the log — names its `logs_index`: `call_result`,
///   `call_error` or `call_interrupted` by their `call`, and
///   `read_result`, `read_error` or `read_interrupted` by their
///   `read`.
///
/// Then what the run's own end keeps is kept, in its order — rule 4
/// of [`lifecycle`](crate::containers::lifecycle): every open ask
/// answered interrupted, in the order asked, where the book keeps
/// asks; `inactive`, where the run was working; `undeployed`, with
/// the provider, the reason [`Reason::DaemonDied`], and `interrupted`
/// true exactly where the run was working. The items are the book's
/// own, made by its [`RunBook`], so a log closed here reads as one
/// closed by its run.
///
/// An agent's log keeps no asks: a structured decision asked and not
/// answered has no interrupted item, and is left as it is.
pub async fn close<B: RunBook>(logs: &Path, book: B) -> Result<Option<Closed>, Error> {
    let dir = book.dir(logs);
    let Some(mut tail) = Tail::open(&dir).await? else {
        return Ok(None);
    };
    let mut active = None;
    let mut answered = HashSet::new();
    let mut open = BTreeMap::new();
    let provider = loop {
        let Some(item) = tail.previous().await? else {
            return Ok(None);
        };
        match item.get("type").and_then(Value::as_str) {
            Some("undeployed") => return Ok(None),
            Some("deployed") => {
                let identity = item.get("identity").cloned().and_then(|identity| serde_json::from_value::<Identity>(identity).ok());
                match identity {
                    Some(identity) => break identity,
                    None => return Err(Error::Deployed(dir.join("log"))),
                }
            }
            Some(kind @ ("active" | "inactive")) => {
                active.get_or_insert(kind == "active");
            }
            Some(kind @ ("call" | "read")) => {
                if let Some(index) = item.get("logs_index").and_then(Value::as_u64) {
                    if !answered.contains(&index) {
                        open.insert(index, if kind == "call" { Ask::Call } else { Ask::Read });
                    }
                }
            }
            Some("call_result" | "call_error" | "call_interrupted") => {
                answered.extend(item.get("call").and_then(Value::as_u64));
            }
            Some("read_result" | "read_error" | "read_interrupted") => {
                answered.extend(item.get("read").and_then(Value::as_u64));
            }
            _ => {}
        }
    };
    drop(tail);
    let active = active.unwrap_or(false);
    for (index, ask) in open {
        if let Some(item) = B::interrupted(index, ask) {
            logs::append(logs, book, item).await?;
        }
    }
    if active {
        logs::append(logs, book, B::inactive()).await?;
    }
    let undeployed = Undeployed {
        provider: Provider { identity: provider.clone() },
        reason: Some(Reason::DaemonDied),
        interrupted: active,
    };
    logs::append(logs, book, B::undeployed(undeployed)).await?;
    Ok(Some(Closed { provider, active }))
}
