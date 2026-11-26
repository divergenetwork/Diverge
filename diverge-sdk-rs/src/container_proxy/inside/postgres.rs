//! Postgres: where the driver dials.
//!
//! The proxy is a database to the program beside it — a pgwire
//! listener on the container's loopback that carries every connection
//! to the caller's own database, raw, one connection one session. No
//! method here speaks the protocol: the program's driver does, as it
//! would to any Postgres, and what it needs from this crate is where
//! — the address, or the whole URL, with the one user every container
//! uses and the name of the database it means.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use serde::{Deserialize, Serialize};

use crate::container_proxy::inside::Client;

/// The port the proxy's pgwire listener is on, inside the container:
/// its own, beside [`INSIDE_PORT`](crate::container_proxy::inside::INSIDE_PORT), because
/// pgwire is not HTTP and cannot share a listener with the paths. The
/// proxy binds it; the program's driver dials it; one copy of the
/// number.
pub const POSTGRES_LOOPBACK_PORT: u16 = 81;

/// The user every container connects as. Not a credential: the
/// caller's side is the authority on what the connection may reach,
/// and passes the name through unread.
pub const POSTGRES_USER: &str = "diverge";

/// Which of its databases a container's connection reaches: what the
/// driver names as the database. Snake case on the wire and in the
/// URL: `"instance"`, `"template"`, `"image"`. The rules, which are
/// the caller's to keep:
///
/// 1. **`instance` is the container's own.** An agent's, a standalone
///    tool's, or a dependency tool's for the one agent it runs for.
///    Nothing else reaches it.
/// 2. **`template` is every container's made from the same
///    template.** An agent template's, a tool template's, or a
///    dependency template's across every agent that declares it.
/// 3. **`image` is every container's of the same image**, by digest.
/// 4. **A dependency tool whose image's `scope` is `global` has no
///    `instance`.** It runs for no one agent, and a connection that
///    names `instance` is refused, as a database that does not exist
///    is.
/// 5. **No other name is a database.** A connection that names
///    anything else is refused the same way.
///
/// One connection reaches one of them; a program may hold any number
/// of connections to each.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "PostgresDatabase")]
#[serde(rename_all = "snake_case")]
pub enum Database {
    /// The container's own: rule 1.
    Instance,
    /// Its template's: rule 2.
    Template,
    /// Its image's: rule 3.
    Image,
}

impl Database {
    /// The name the driver dials: `instance`, `template` or `image`.
    pub const fn name(self) -> &'static str {
        match self {
            Database::Instance => "instance",
            Database::Template => "template",
            Database::Image => "image",
        }
    }

    /// The database a name is, or `None` for a name that is none of
    /// the three: rule 5.
    pub fn named(name: &str) -> Option<Database> {
        match name {
            "instance" => Some(Database::Instance),
            "template" => Some(Database::Template),
            "image" => Some(Database::Image),
            _ => None,
        }
    }
}

impl Client {
    /// Where to point the driver: the proxy's pgwire listener on the
    /// container's loopback.
    ///
    /// Every connection the driver opens here is one session with one
    /// of the container's databases — the one the driver names, see
    /// [`Database`] — opened as the driver asks and held until either
    /// side hangs up; a pool is the ordinary case. The proxy passes
    /// everything through unread, TLS negotiation included. A session
    /// that ended is a hang-up the driver already knows how to report,
    /// and the proxy retries nothing: a socket cannot be resumed, and
    /// a pool reconnects on its own.
    pub fn postgres_address(&self) -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::LOCALHOST,
            POSTGRES_LOOPBACK_PORT,
        ))
    }

    /// The whole URL of one of the container's databases, for a driver
    /// that takes one:
    /// `postgres://diverge@127.0.0.1:81/instance?sslmode=disable`.
    ///
    /// The user is [`POSTGRES_USER`], fixed, with no password — a
    /// container has no credential of its own to offer — and the
    /// database is the [`Database`]'s name. TLS is off, because the
    /// proxy is loopback and the conduit beyond it is the provider's
    /// to secure; a driver that negotiated TLS here would be
    /// negotiating with whatever the caller routed the bytes to.
    pub fn postgres_url(&self, database: Database) -> String {
        format!(
            "postgres://{POSTGRES_USER}@127.0.0.1:{POSTGRES_LOOPBACK_PORT}/{}?sslmode=disable",
            database.name(),
        )
    }
}
