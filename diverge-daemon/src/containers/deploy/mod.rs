//! An agent's declared dependencies, deployed then and there.
//!
//! An agent's program answers its registration with the tools it
//! depends on, each a dependency tool template, and the provider asks
//! the daemon to deploy them before the container's id is out, the id
//! told with the ask. [`deploy`] starts every one at once, each a tool
//! container of the agent's own: on any connected provider, tried in
//! random order; its mounts the agent's paths served live from the
//! agent's container; its secrets the agent's own, by the names its
//! image states, bound through the agent's binding and taken at
//! the dependency's own registration, held and written back as any
//! tool container's; its limits what its image needs and what its
//! template adds; its
//! database scope the one its image's `scope` names; its requests to the
//! daemon judged by the grants its own program declared at its
//! registration; served to the agent under
//! its template's id, short. A template declared twice, one of an
//! image that is no dependency tool's, one naming a mount of an image
//! that is not lazy and of the `instance` scope, or one of an image
//! needing a secret the agent does not bind, is refused. The first that
//! cannot be deployed — an eager tool whose start fails included — is the
//! run's error, and the rest are stopped. A dependency lives for as
//! long as the agent's container does, and no longer. A tool
//! container declares no dependencies and is never deployed for.
//!
//! Its own files are flattened into it, so everything is named
//! through this module and not through the file it lives in.

mod alone;
mod deploy;

pub use alone::*;
pub use deploy::*;
