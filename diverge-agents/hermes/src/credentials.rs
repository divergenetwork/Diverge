//! The logins that refresh, kept in the program's own database.
//!
//! A credential is an argument, and the arguments never change. A
//! login whose tokens are single-use does: Hermes rewrites its
//! document at every refresh, and the document the arguments gave is
//! then a login already spent. So the program keeps each login as the
//! last run left it — one row for each, in the same database the
//! [`continuation`](crate::continuation) is kept in, the agent's
//! `instance` database — and every run starts from what is kept.
//!
//! # Which document a run uses
//!
//! 1. A row is kept under the login's argument — `provider.oauth` or
//!    `toolsets.spotify.oauth` — with the SHA-256 of the canonical
//!    bytes of the document the arguments gave when the row was
//!    written, and the document as the run left it.
//! 2. A run uses the kept document when the row's hash is the hash
//!    of the document its own arguments give: it was kept for this
//!    same argument value.
//! 3. Otherwise — no row, or a row kept for another value — the run
//!    uses the document its arguments give.
//! 4. After the gateway has exited, each document that is still on
//!    disk is kept, in place of the row, under the hash of the
//!    document the run's arguments give. A document no longer on
//!    disk changes no row.
//!
//! Rule 3 is what lets a login be replaced: an agent made anew with
//! another document under the same argument starts from that
//! document, and the next keep replaces the row.
//!
//! Nothing here carries a document or any part of one into an error:
//! a failure is the database's own, and names no value.

use diverge_sdk::shared::canonical;
use serde_json::{Map, Value};
use sha2::{Digest as _, Sha256};
use sqlx::{PgPool, Row as _};

use crate::filesystem::Document;

/// The table: one login to a row.
const CREATE: &str = "CREATE TABLE IF NOT EXISTS credentials (\
    argument text PRIMARY KEY, given bytea NOT NULL, login text NOT NULL)";

/// One login's row.
const SELECT: &str = "SELECT given, login FROM credentials WHERE argument = $1";

/// One login, in place of its row.
const KEEP: &str = "INSERT INTO credentials (argument, given, login) VALUES ($1, $2, $3) \
    ON CONFLICT (argument) DO UPDATE SET given = EXCLUDED.given, login = EXCLUDED.login";

/// The document to use for each of `documents`, in the same order:
/// rules 2 and 3. The table is made if this is the first run to
/// look; an agent with no login asks the database nothing.
pub async fn resolve(
    pool: &PgPool,
    documents: &[Document],
) -> Result<Vec<Map<String, Value>>, Error> {
    if documents.is_empty() {
        return Ok(Vec::new());
    }
    sqlx::query(CREATE).execute(pool).await?;
    let mut logins = Vec::with_capacity(documents.len());
    for document in documents {
        let given = hash(&document.login)?;
        let row = sqlx::query(SELECT)
            .bind(document.argument)
            .fetch_optional(pool)
            .await?;
        let kept = match row {
            Some(row) if row.get::<Vec<u8>, _>(0) == given => {
                match serde_json::from_str::<Value>(row.get::<&str, _>(1))? {
                    Value::Object(login) => Some(login),
                    _ => None,
                }
            }
            _ => None,
        };
        logins.push(kept.unwrap_or_else(|| document.login.clone()));
    }
    Ok(logins)
}

/// Keep each document the run left: rule 4. `left` has one place for
/// each of `documents`, in the same order, as
/// [`read_back`](crate::filesystem::read_back) answers it.
pub async fn keep(
    pool: &PgPool,
    documents: &[Document],
    left: Vec<Option<Map<String, Value>>>,
) -> Result<(), Error> {
    for (document, login) in documents.iter().zip(left) {
        let Some(login) = login else {
            continue;
        };
        sqlx::query(KEEP)
            .bind(document.argument)
            .bind(hash(&document.login)?)
            .bind(serde_json::to_string(&login)?)
            .execute(pool)
            .await?;
    }
    Ok(())
}

/// The SHA-256 of a document's canonical bytes: what a row is kept
/// under, so that two spellings of one document are one value.
fn hash(login: &Map<String, Value>) -> Result<Vec<u8>, serde_json::Error> {
    Ok(Sha256::digest(canonical::bytes(login)?).to_vec())
}

/// The logins could not be read from the database, or kept in it.
#[derive(Debug)]
pub enum Error {
    /// The database would not answer.
    Database(sqlx::Error),
    /// A document would not serialize, or a kept one would not parse.
    /// The message is the parser's, which quotes no value.
    Json(serde_json::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Database(error) => {
                write!(f, "the credentials' database failed: {error}")
            }
            Error::Json(error) => {
                write!(f, "a kept login could not be read or written: {error}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Database(error) => Some(error),
            Error::Json(error) => Some(error),
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Error::Database(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::Json(error)
    }
}

impl Error {
    /// The failure as JSON, the shape every failure on the stream has.
    pub fn message(&self) -> serde_json::Value {
        serde_json::json!({
            "kind": "credentials",
            "error": self.to_string(),
        })
    }
}
