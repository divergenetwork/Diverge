//! Accounts: listing a template's, adding and removing one, one's
//! username, and one's token, good now.
//!
//! An account is one user of one issuer — `<issuer>.<id>` — and at
//! `identity.diverge.network` it has a **username**, which it sets
//! once and never changes: the first part of every subject of its,
//! and its namespace at the registry. The rules of the username here:
//!
//! 1. **Kept once learned.** A row's `username` is null until
//!    `identity.diverge.network` says the account has one, and is
//!    never asked for again once it does: a username does not change,
//!    so what was learned stays true.
//! 2. **Learned by asking, as the account** — see [`learn`]. An
//!    account that cannot be asked — signed out at its issuer — keeps
//!    the null it has, and is asked again the next time.
//! 3. **How a caller names an account.** A tool call names an account
//!    by its username — see [`named`] — and so can name only one that
//!    has set one. An account with none is named by nobody but the
//!    page, by its issuer and id, where a person sets it — see
//!    [`set_username`].

use std::time::{SystemTime, UNIX_EPOCH};

use diverge_sdk::identity_diverge_network::{self, Error, Issuer, SERVICE, Tokens, Which};
use futures_util::future::join_all;
use serde::Serialize;
use sqlx::{PgPool, Row as _};

use crate::failure::Failure;

/// One account, as the tool lists it: no token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Account {
    /// Its username at `identity.diverge.network`, or null where it
    /// has set none, or could not be asked.
    pub username: Option<String>,
    /// The account, written `<issuer>.<id>`.
    pub account: String,
    /// Its issuer's name.
    pub issuer: String,
    /// Its id at that issuer.
    pub id: String,
}

/// An account a caller named by its username: who it is.
#[derive(Debug, Clone)]
pub struct Named {
    /// Its issuer.
    pub issuer: &'static Issuer,
    /// Its id at that issuer.
    pub id: String,
}

/// Now, in seconds since the Unix epoch.
fn now() -> i64 {
    let seconds = SystemTime::now().duration_since(UNIX_EPOCH).map(|since| since.as_secs()).unwrap_or(0);
    i64::try_from(seconds).unwrap_or(i64::MAX)
}

/// Every account kept under `template`, by issuer and then id, as the
/// rows have them: no token is read, and nobody is asked.
async fn rows(pool: &PgPool, template: &str) -> Result<Vec<Account>, Failure> {
    let rows = sqlx::query("select issuer, id, username from identity_accounts where template = $1 order by issuer, id").bind(template).fetch_all(pool).await?;
    let mut accounts = Vec::with_capacity(rows.len());
    for row in rows {
        let (issuer, id): (String, String) = (row.try_get("issuer")?, row.try_get("id")?);
        accounts.push(Account { username: row.try_get("username")?, account: format!("{issuer}.{id}"), issuer, id });
    }
    Ok(accounts)
}

/// Every account kept under `template`, by issuer and then id, each
/// with its username. One whose username is not kept yet is asked for
/// it — rule 2 — all of them at once; one that has set none, or cannot
/// be asked, is listed with null, and that it could not be asked is no
/// failure of the list.
pub async fn list(pool: &PgPool, http: &reqwest::Client, template: &str) -> Result<Vec<Account>, Failure> {
    let mut accounts = rows(pool, template).await?;
    let asked = join_all(accounts.iter().map(|account| async move {
        match (&account.username, identity_diverge_network::issuer(&account.issuer)) {
            (None, Ok(issuer)) => learn(pool, http, template, issuer, &account.id).await.ok().flatten(),
            _ => None,
        }
    }))
    .await;
    for (account, learned) in accounts.iter_mut().zip(asked) {
        if account.username.is_none() {
            account.username = learned;
        }
    }
    Ok(accounts)
}

/// Keep the account `id` at `issuer` under `template`, with `tokens`:
/// a new row, or — an account signed in again — the row it has, its
/// tokens replaced by these and its username kept.
pub async fn add(pool: &PgPool, template: &str, issuer: &str, id: &str, tokens: &Tokens) -> Result<(), Failure> {
    sqlx::query(
        "insert into identity_accounts (template, issuer, id, access_token, access_expires, refresh_token, id_token)
         values ($1, $2, $3, $4, $5, $6, $7)
         on conflict (template, issuer, id) do update
         set access_token = excluded.access_token, access_expires = excluded.access_expires,
             refresh_token = excluded.refresh_token, id_token = excluded.id_token",
    )
    .bind(template)
    .bind(issuer)
    .bind(id)
    .bind(&tokens.access_token)
    .bind(tokens.access_expires)
    .bind(&tokens.refresh_token)
    .bind(&tokens.id_token)
    .execute(pool)
    .await?;
    Ok(())
}

/// Forget the account `id` at `issuer` kept under `template`: whether
/// there was one. Its tokens are deleted here and ended nowhere: what
/// it signed in on a daemon stays signed in, the account's own
/// sign-in at its issuer stands until it is ended there, and its
/// username is its own still.
pub async fn remove(pool: &PgPool, template: &str, issuer: &str, id: &str) -> Result<bool, Failure> {
    let removed = sqlx::query("delete from identity_accounts where template = $1 and issuer = $2 and id = $3").bind(template).bind(issuer).bind(id).execute(pool).await?;
    Ok(removed.rows_affected() > 0)
}

/// Keep `username` as the username of the account `id` at `issuer`
/// kept under `template` — rule 1.
async fn keep(pool: &PgPool, template: &str, issuer: &Issuer, id: &str, username: &str) -> Result<(), Failure> {
    sqlx::query("update identity_accounts set username = $4 where template = $1 and issuer = $2 and id = $3").bind(template).bind(issuer.name).bind(id).bind(username).execute(pool).await?;
    Ok(())
}

/// Ask `identity.diverge.network` the username of the account `id` at
/// `issuer` kept under `template`, as the account, and keep it where
/// it has one — rules 1 and 2. `None` where the account has set none.
pub async fn learn(pool: &PgPool, http: &reqwest::Client, template: &str, issuer: &'static Issuer, id: &str) -> Result<Option<String>, Failure> {
    let token = token(pool, http, template, issuer, id, issuer.presents).await?;
    let username = identity_diverge_network::username(http, SERVICE, issuer, &token).await.map_err(|error| Failure::Identity(error.to_string()))?;
    if let Some(username) = &username {
        keep(pool, template, issuer, id, username).await?;
    }
    Ok(username)
}

/// The account kept under `template` whose username is `username` —
/// rule 3. In order:
///
/// 1. **The row that keeps it** is the account.
/// 2. **No row keeps it:** every account with no username kept is
///    asked for one — [`list`] — since one of them may have set it
///    since, elsewhere; and the row that keeps it then is the account.
/// 3. **Still none** is [`Failure::Unknown`].
pub async fn named(pool: &PgPool, http: &reqwest::Client, template: &str, username: &str) -> Result<Named, Failure> {
    let found = |accounts: Vec<Account>| accounts.into_iter().find(|account| account.username.as_deref() == Some(username));
    // Rule 1.
    let account = match found(rows(pool, template).await?) {
        Some(account) => account,
        // Rules 2 and 3.
        None => found(list(pool, http, template).await?).ok_or_else(|| Failure::Unknown(username.to_string()))?,
    };
    let issuer = identity_diverge_network::issuer(&account.issuer).map_err(|error| Failure::Issuer(error.to_string()))?;
    Ok(Named { issuer, id: account.id })
}

/// Set the username of the account `id` at `issuer` kept under
/// `template`, at `identity.diverge.network`, as the account, and keep
/// it. Once: an account that has one already is
/// [`Failure::UsernameSet`], with the one it has, which is kept; a
/// username that is another account's is [`Failure::UsernameTaken`];
/// one that may be none is [`Failure::Parameters`], with the rule as
/// `identity.diverge.network` says it.
pub async fn set_username(pool: &PgPool, http: &reqwest::Client, template: &str, issuer: &'static Issuer, id: &str, username: &str) -> Result<String, Failure> {
    let token = token(pool, http, template, issuer, id, issuer.presents).await?;
    match identity_diverge_network::set_username(http, SERVICE, issuer, &token, username).await {
        Ok(username) => {
            keep(pool, template, issuer, id, &username).await?;
            Ok(username)
        }
        Err(Error::Service { code, description, .. }) => match code.as_str() {
            "username_taken" => Err(Failure::UsernameTaken(username.to_string())),
            "username_set" => {
                let has = learn(pool, http, template, issuer, id).await?.unwrap_or_default();
                Err(Failure::UsernameSet(has))
            }
            "invalid_request" => Err(Failure::Parameters(description.unwrap_or(code))),
            _ => Err(Failure::Identity(format!("identity.diverge.network refused, `{code}`: {}", description.unwrap_or_default()))),
        },
        Err(error) => Err(Failure::Identity(error.to_string())),
    }
}

/// The token, `which` of the two, of the account `id` at `issuer`
/// kept under `template`, good to use now. One transaction, the row
/// locked in it, which is this store's turn — what a file's lock is to
/// an account kept in one. In order:
///
/// 1. **The row is read, and locked**: `for update`, so that of two
///    calls that find a token expired, one renews it and the other
///    waits, and reads what the first wrote. No such row is
///    [`Failure::NoAccount`].
/// 2. **A token still good is answered** as it is — see
///    [`Tokens::good`] — and the transaction ended.
/// 3. **Any other is renewed** — see [`Tokens::renew`] — the row
///    written, and the transaction committed, before the token is
///    answered: the lock is let go before the request the token is
///    for.
/// 4. **Signed out.** A renewal the issuer refuses, and one that
///    answers a token no better, are [`Failure::SignedOut`], and the
///    row is left as it was.
pub async fn token(pool: &PgPool, http: &reqwest::Client, template: &str, issuer: &Issuer, id: &str, which: Which) -> Result<String, Failure> {
    let account = format!("{}.{id}", issuer.name);
    let mut transaction = pool.begin().await?;
    // Rule 1.
    let row = sqlx::query("select access_token, access_expires, refresh_token, id_token from identity_accounts where template = $1 and issuer = $2 and id = $3 for update")
        .bind(template)
        .bind(issuer.name)
        .bind(id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or_else(|| Failure::NoAccount(account.clone()))?;
    let mut tokens = Tokens {
        access_token: row.try_get("access_token")?,
        access_expires: row.try_get("access_expires")?,
        refresh_token: row.try_get("refresh_token")?,
        id_token: row.try_get("id_token")?,
    };
    // Rule 2.
    if let Some(token) = tokens.good(which, now()) {
        return Ok(token.to_string());
    }
    // Rules 3 and 4.
    tokens.renew(http, issuer, now()).await.map_err(|error| match error {
        Error::Refused { .. } => Failure::SignedOut(account.clone()),
        error => Failure::Identity(error.to_string()),
    })?;
    let token = tokens.good(which, now()).map(str::to_string).ok_or_else(|| Failure::SignedOut(account.clone()))?;
    sqlx::query("update identity_accounts set access_token = $4, access_expires = $5, refresh_token = $6, id_token = $7 where template = $1 and issuer = $2 and id = $3")
        .bind(template)
        .bind(issuer.name)
        .bind(id)
        .bind(&tokens.access_token)
        .bind(tokens.access_expires)
        .bind(&tokens.refresh_token)
        .bind(&tokens.id_token)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    Ok(token)
}
