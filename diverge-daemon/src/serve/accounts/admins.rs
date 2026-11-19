//! The administrators the configuration names, listed as accounts.
//!
//! The rules:
//!
//! 1. Every identity the daemon's `admins` names that no account is,
//!    is listed as an account with `admin` true, its tags, no
//!    description or roles, and no `created` or `creator`: the
//!    configuration made it, not a request.
//! 2. An identity `admins` names that an account is, is that account
//!    and no administrator: it is listed once, as the account, with
//!    `admin` false, and holds what its roles grant.
//! 3. `connected` is whether a client is connected as it now,
//!    administrators counted as accounts are; `last_connected` is when
//!    that last changed, known only since the daemon started.
//! 4. A grant's `within` and a request's filter read an administrator by
//!    the facts it has — its identity, its tags, whether it is
//!    connected — and never let it through a filter on roles, creators
//!    or times made, which it does not have.
//! 5. The configuration is read once: an administrator's entry changes
//!    only as its `connected` does.
//! 6. It is tagged and untagged as an account is, its tags kept by
//!    its identity. It is not stored, so it is neither edited nor
//!    deleted: those requests answer `Administrator`. It is not
//!    created either: it is there, and a create of it answers
//!    `Exists`. Its command log is read as an account's, and as the
//!    `admin` subject's.

use diverge_sdk::daemon::endpoints::accounts::list::client::request::Filter;
use diverge_sdk::daemon::endpoints::accounts::list::server::response::Account;
use diverge_sdk::daemon::grant::accounts::Over;
use chrono::{DateTime, Utc};
use diverge_sdk::shared::Identity;

use crate::daemon::Daemon;
use crate::judge::{self, Standing, filter};
use crate::store::{self, accounts};

/// Rule 1: an administrator's entry, with its tags, whether a client
/// is connected as it and when that last changed.
pub fn entry(identity: &Identity, tags: Vec<String>, connected: bool, last_connected: Option<DateTime<Utc>>) -> Account {
    Account {
        identity: identity.clone(),
        description: None,
        roles: Vec::new(),
        connected,
        last_connected,
        tags,
        created: None,
        creator: None,
        admin: true,
    }
}

/// Rules 1 to 4: the administrators `standing` may do `action` to and
/// `filter`, when given, lets through, but for those `rows` already
/// hold (rule 2), each as its entry.
pub async fn entries(daemon: &Daemon, standing: &Standing, action: Over, filter: Option<&Filter>, rows: &[accounts::Account]) -> Result<Vec<Account>, store::Error> {
    let connected = daemon.live.admins_connected().await;
    let mut tags = {
        let mut conn = daemon.store.acquire().await?;
        store::admins::all(&mut conn).await?
    };
    let admitted = daemon
        .authenticator
        .admins()
        .filter(|identity| !rows.iter().any(|row| &row.identity == *identity))
        .map(|identity| (identity, tags.remove(identity).unwrap_or_default(), connected.contains(identity)))
        .filter(|(identity, tags, connected)| judge::accounts::over_admin(standing, action, identity, tags, *connected))
        .filter(|(identity, tags, connected)| filter.is_none_or(|filter| filter::accounts::test_admin(filter, identity, tags, *connected)))
        .collect::<Vec<_>>();
    let mut entries = Vec::new();
    for (identity, tags, connected) in admitted {
        entries.push(entry(identity, tags, connected, daemon.live.admin_last_connected(identity).await));
    }
    Ok(entries)
}
