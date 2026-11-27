//! The `auth_file` login as the loops leave it, kept in the agent's
//! own database.
//!
//! Codex rewrites `auth.json` in place when a ChatGPT login
//! refreshes, and a refreshed login is the only one that still logs
//! in: the tokens the argument carries are spent by the first
//! refresh. So what a loop leaves of the file is kept, one row in the
//! agent's `instance` database, beside the continuation and through
//! the same pool, and the next loop — or the next start of the
//! container — writes that and not the argument.
//!
//! The rules:
//!
//! 1. The row holds the login, and the SHA-256 of the canonical bytes
//!    of the `auth` argument it was kept for, as lowercase
//!    hexadecimal. The argument itself is never stored, and nothing
//!    else of it is.
//! 2. At a loop's start, a row whose hash is the hash of the agent's
//!    `auth` argument is the login the loop is handed: the kept one.
//! 3. No row, or a row whose hash is another's — the agent was made
//!    anew with another `auth` — hands the loop the argument, and the
//!    row is replaced with it under its own hash.
//! 4. After a loop's last turn, `auth.json` as Codex left it replaces
//!    the row's login, under the same hash.
//! 5. The other three methods keep nothing, and never read the row.
//!
//! No value of a login is ever formatted into an error or a log.

use std::fmt;

use serde_json::{Map, Value};
use sha2::{Digest as _, Sha256};
use sqlx::{PgPool, Row as _};

const CREATE: &str = "CREATE TABLE IF NOT EXISTS codex_login (\
    id smallint PRIMARY KEY CHECK (id = 1), \
    argument text NOT NULL, \
    auth jsonb NOT NULL)";
const SELECT: &str = "SELECT argument, auth::text FROM codex_login WHERE id = 1";
const UPSERT: &str = "INSERT INTO codex_login (id, argument, auth) \
    VALUES (1, $1, CAST($2 AS jsonb)) \
    ON CONFLICT (id) DO UPDATE SET argument = EXCLUDED.argument, \
    auth = EXCLUDED.auth";

/// The argument's hash: rule 1.
fn hash(argument: &Map<String, Value>) -> Result<String, Error> {
    let bytes = diverge_sdk::shared::canonical::bytes(argument).map_err(Error::Json)?;
    let mut hash = String::with_capacity(64);
    for byte in Sha256::digest(&bytes) {
        hash.push_str(&format!("{byte:02x}"));
    }
    Ok(hash)
}

/// The login a loop is handed, by rules 2 and 3: the kept one where
/// it was kept for `argument`, else `argument`, the row replaced.
pub async fn current(pool: &PgPool, argument: &Map<String, Value>) -> Result<Map<String, Value>, Error> {
    let hash = hash(argument)?;
    sqlx::query(CREATE).execute(pool).await.map_err(Error::Database)?;
    if let Some(row) = sqlx::query(SELECT).fetch_optional(pool).await.map_err(Error::Database)? {
        let kept_for: String = row.get(0);
        if kept_for == hash {
            let kept: String = row.get(1);
            return serde_json::from_str(&kept).map_err(Error::Json);
        }
    }
    write(pool, &hash, argument).await?;
    Ok(argument.clone())
}

/// Keep what a loop left, by rule 4: `left` in the row, under the
/// hash of `argument`.
pub async fn keep(pool: &PgPool, argument: &Map<String, Value>, left: &Map<String, Value>) -> Result<(), Error> {
    let hash = hash(argument)?;
    write(pool, &hash, left).await
}

/// The row, replaced.
async fn write(pool: &PgPool, hash: &str, login: &Map<String, Value>) -> Result<(), Error> {
    let login = serde_json::to_string(login).map_err(Error::Json)?;
    sqlx::query(UPSERT).bind(hash).bind(login).execute(pool).await.map_err(Error::Database)?;
    Ok(())
}

/// The kept login could not be read or written.
#[derive(Debug)]
pub enum Error {
    /// The database would not answer.
    Database(sqlx::Error),
    /// A login would not serialize, or the row's is not a JSON
    /// object. The message is the parser's, which quotes no value.
    Json(serde_json::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Database(error) => write!(f, "the kept login's database would not answer: {error}"),
            Error::Json(_) => f.write_str("the kept login is not a JSON object"),
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

impl Error {
    /// The failure as JSON, the shape every failure on the stream
    /// has.
    pub fn message(&self) -> serde_json::Value {
        serde_json::json!({
            "kind": "login",
            "error": self.to_string(),
        })
    }
}
