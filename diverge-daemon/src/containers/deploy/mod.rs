//! An agent's declared dependencies, deployed then and there.
//!
//! An agent's program answers its registration with the tools it
//! depends on, each a dependency tool template, and the provider asks
//! the daemon to deploy them before the container's id is out, the id
//! told with the ask. [`deploy`] records every one, and deploys those
//! that are the agent's own or eager, each a tool
//! container: on any connected provider, tried in
//! random order; its mounts the agent's paths served live from the
//! agent's container; its limits what its image needs and what its
//! template adds; its
//! compartments of the database the three every container has, less its own where its image's `scope` is `template`; its requests to the
//! daemon judged by the grants its own program declared at its
//! registration; served to the agent under
//! its template's id, short. A template declared twice, one of an
//! image that is no dependency tool's, one naming a mount of an image
//! that is not lazy and of the `instance` scope, is
//! refused. The first that
//! cannot be deployed — an eager tool whose start fails included — is the
//! run's error. A dependency's run, once up, is held and undeployed as
//! every tool's is — see [`idle_tool`](crate::containers::idle_tool) —
//! and one that mounts its agent's files goes down with its agent's
//! container. [`tool`] is one run of any dependency tool, which every
//! deploy of one comes to. [`served`] deploys every tool an agent is
//! served that is down, of every kind, all at once: what the agent's
//! start does once its registration is over, and what a message does
//! before it is delivered. A tool
//! container declares no dependencies and is never deployed for.
//!
//! Its own files are flattened into it, so everything is named
//! through this module and not through the file it lives in.

mod deploy;
mod served;
mod tool;

pub use deploy::*;
pub use served::*;
pub use tool::*;
