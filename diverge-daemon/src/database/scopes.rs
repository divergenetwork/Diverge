//! A compartment as it is live: the password in memory, and the role
//! made once per daemon life.

use std::sync::Arc;

use diverge_sdk::container_proxy::inside::Database;
use diverge_sdk::daemon::endpoints::postgres::compartments::Compartment;
use rand::Rng as _;
use sqlx::PgConnection;
use tokio::sync::OnceCell;

use super::{provision, reached, role_of};
use crate::containers::Key;
use crate::daemon::{Daemon, Kind};
use crate::store::{self, postgres_compartments};

/// One compartment's role, live: its name, the password the daemon
/// minted for this daemon's life, and whether the role has been made —
/// or found made and given this password — since the start. One per
/// role, whichever container asks: a role two containers reach has one
/// password, or each would take the other's away.
#[derive(Debug)]
pub struct Scope {
    /// The role, and the schema.
    pub role: String,
    /// The password, hex of thirty-two random bytes; in memory only.
    pub password: String,
    ready: OnceCell<()>,
}

impl Scope {
    /// A role not yet made this daemon life, with a fresh password.
    pub fn new(role: String) -> Scope {
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        Scope {
            role,
            password: hex::encode(bytes),
            ready: OnceCell::new(),
        }
    }
}

/// Why a container was given no compartment.
#[derive(Debug)]
pub enum Refusal {
    /// The container has no such database: one whose image's scope is
    /// `global` named `instance`. Told as Postgres tells a database
    /// that does not exist.
    Absent(String),
    /// The compartment could not be made: a record gone, a role that
    /// cannot provision, the store.
    Failed(String),
}

/// The compartment the container's connection reaches, made if it was
/// not this daemon life: what the container reaches looked up, the
/// compartment the database names taken — or refused, where the
/// container has none — the daemon's role checked able, every
/// compartment of the container put on record, and the statements run
/// once: a pool opening many connections at once waits on the first.
pub async fn scope(daemon: &Daemon, key: Key, database: Database) -> Result<(Arc<Scope>, Compartment), Refusal> {
    let failed = |error: store::Error| Refusal::Failed(error.to_string());
    let reached = {
        let mut conn = daemon.store.acquire().await.map_err(failed)?;
        reached(&mut conn, key)
            .await
            .map_err(failed)?
            .ok_or_else(|| Refusal::Failed("the container's record is gone".to_string()))?
    };
    let Some(compartment) = reached.of(database).cloned() else {
        return Err(Refusal::Absent(format!(
            "database \"{}\" does not exist: a dependency tool whose image's scope is `global` runs for no one agent, and has `template` and `image` alone",
            database.name()
        )));
    };
    let scope = daemon.live.scope(role_of(&compartment)).await;
    scope
        .ready
        .get_or_try_init(|| async {
            let mut tx = daemon.store.begin().await.map_err(|error| error.to_string())?;
            provision::capable(&mut tx).await.map_err(|error| error.to_string())??;
            for compartment in reached.all() {
                postgres_compartments::register(&mut tx, compartment, reached.kind)
                    .await
                    .map_err(|error| error.to_string())?;
            }
            provision::ensure(&mut tx, &scope.role, &scope.password)
                .await
                .map_err(|error| error.to_string())?;
            tx.commit().await.map_err(|error| error.to_string())?;
            daemon.live.changed(Kind::PostgresCompartments);
            Ok::<(), String>(())
        })
        .await
        .map_err(Refusal::Failed)?;
    Ok((scope, compartment))
}

/// Put every compartment the container reaches on record, in the
/// caller's transaction: what the making of an agent, a tool or a
/// dependency tool does, so that a compartment is listed from the
/// moment what it is of is there. One already on record is left as it
/// is.
pub async fn register(conn: &mut PgConnection, key: Key) -> Result<(), store::Error> {
    if let Some(reached) = reached(conn, key).await? {
        for compartment in reached.all() {
            postgres_compartments::register(conn, compartment, reached.kind).await?;
        }
    }
    Ok(())
}

/// Remove the compartment, in the caller's transaction: its role and
/// its schema dropped with every table, its record gone, its password
/// forgotten. One that was never made, or is on no record, is nothing
/// to do.
pub async fn remove(daemon: &Daemon, conn: &mut PgConnection, compartment: &Compartment) -> Result<(), store::Error> {
    let role = role_of(compartment);
    provision::drop(conn, &role).await?;
    postgres_compartments::delete(conn, &role).await?;
    daemon.live.forget_scope(&role).await;
    Ok(())
}
