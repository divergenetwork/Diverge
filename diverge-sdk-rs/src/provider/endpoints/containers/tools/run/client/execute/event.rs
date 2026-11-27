//! One thing the tool's main stream says: the proxy's word that the
//! container went active, or inactive.

/// What a reader of a tool container's main stream gets, one at a
/// time: the two things it carries after its opening, as
/// [`Frame`](crate::provider::endpoints::containers::tools::run::server::response::Frame)
/// carries them — the proxy's own [`Active`](Self::Active) when an
/// MCP exchange is in flight where none was, and
/// [`Inactive`](Self::Inactive) when the last in flight has been
/// answered.
///
/// One type for both layers: the provider reads it off the proxy's
/// begin stream, and the caller reads it off the run's, so a relay
/// between the two is a match with two arms and no translation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// An exchange is in flight where none was: the container is
    /// serving until the matching `Inactive`.
    Active,
    /// The last exchange in flight has been answered. The container
    /// is idle until the next exchange.
    Inactive,
}
