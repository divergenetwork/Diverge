//! The compartments of the database: the three a container reaches,
//! listed, tagged and deleted.
//!
//! A container's driver names one of three databases, `instance`,
//! `template` or `image`, and each is a [`Compartment`]: one login
//! role and one schema of the same name in the daemon's one database,
//! reached by every container the compartment is of and by no other.
//! A compartment is on record from the moment what it is of is made —
//! an agent, a tool, a dependency tool — and its role and schema are
//! made at the first connection that names it.
//!
//! # How each goes
//!
//! 1. **An agent's or a tool's `instance`** goes with the agent's or
//!    the tool's delete.
//! 2. **An agent's or a tool's `template`** goes with the template's
//!    delete.
//! 3. **A dependency tool's `instance`** goes with its agent's delete.
//! 4. **A dependency tool's `template`** goes by [`delete`] alone, and
//!    only while no agent's last registration declared the dependency
//!    template. The delete removes the template's every tool with it,
//!    each with its own `instance`, and the template's body, so that a
//!    registration declaring it anew makes it anew.
//! 5. **An `image`** goes by [`delete`] alone: an agent's or a
//!    standalone tool's image's only while no template there is
//!    contains the image; a dependency tool's image's only while no
//!    agent's last registration declared a dependency template of the
//!    image.
//!
//! Whether a [`delete`] would be allowed now is each item's
//! `deletable`: `false` for the compartments of rules 1 to 3, always.
//! A delete of a compartment a container holds a connection through,
//! or whose dependency tool runs, is `InUse`.
//!
//! [`list`] lists them, narrowed; [`get`] answers one; [`tag`] and
//! [`untag`] change one's tags; [`delete`] removes one.

mod compartment;

pub use compartment::*;

pub mod delete;
pub mod get;
pub mod list;
pub mod tag;
pub mod untag;
