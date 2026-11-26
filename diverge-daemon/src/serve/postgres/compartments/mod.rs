//! The five requests over the database's compartments, served:
//! [`list`], [`get`], [`tag`], [`untag`] and [`delete`].
//!
//! A request names a compartment as the wire does, an agent in it
//! either way; [`find`] is that compartment's record, the agent named
//! by its template and index first, and [`deletable`] whether a delete
//! of it would be allowed now.

use diverge_sdk::daemon::endpoints::postgres::compartments::Compartment;
use sqlx::PgConnection;

use crate::database;
use crate::store::postgres_compartments::{self, Entry, Holds};
use crate::store;

pub mod delete;
pub mod get;
pub mod list;
pub mod tag;
pub mod untag;

/// The record of the compartment a request names, if it is on record.
/// With `lock`, its row is locked for the transaction.
pub async fn find(conn: &mut PgConnection, compartment: &Compartment, lock: bool) -> Result<Option<Entry>, store::Error> {
    let Some(compartment) = database::canonical_of(conn, compartment).await? else {
        return Ok(None);
    };
    postgres_compartments::by_role(conn, &database::role_of(&compartment), lock).await
}

/// Whether a delete of the compartment would be allowed now,
/// connections apart.
pub async fn deletable(conn: &mut PgConnection, entry: &Entry) -> Result<bool, store::Error> {
    Ok(Holds::load(conn).await?.deletable(entry))
}
