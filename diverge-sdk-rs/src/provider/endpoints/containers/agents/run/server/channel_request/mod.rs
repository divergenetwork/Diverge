//! The channels a provider opens on a caller during a run.
//!
//! Twenty-two: six the provider asks on its own account, sixteen it
//! relays from the container. The tools family has the same six of
//! its own — its registration carries no dependencies, which a tool
//! container is never asked about — and eleven relayed. See
//! [`Frame`].

mod frame;

pub use frame::*;
