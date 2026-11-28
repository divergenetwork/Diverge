//! Eager tools: the tools an attachment keeps.
//!
//! An eager tool is a tool whose image's `mode` is `eager`: a
//! dependency tool — one row under its scope key, the agent or the
//! template alone, made by the first registration that declares it
//! there — or a tool on record. It is deployed, held and undeployed as
//! every tool is, by the one set of holds
//! [`idle_tool`](crate::containers::idle_tool) states, and `eager` is
//! exactly one hold more: any agent ATTACHED to it — a dependency
//! tool's by the registrations that declare its template, a tool on
//! record's by `attach`, to an agent or to an agent template, which
//! attaches it to every agent made from the template — whether or not
//! that agent runs. While an agent is attached the tool is KEPT: its
//! container started at the attach or the registration that first
//! held it, started again whenever its run ends, and started at every
//! boot of the daemon's. Once none is attached it is kept no more,
//! and its run is its idle clock's like any other: undeployed
//! `idle_seconds` after the last hold left it, deployed again by the
//! next thing that uses it, and kept again by the next attach. A
//! dependency tool's row goes with its agent's delete, or with the
//! delete of its template's compartment of the database; a tool on
//! record's with the tool's delete; either stops its run.
//!
//! [`held`] is the one way in: a tool's attachments read anew, and the
//! tool kept or kept no more by them — its [`State`] made and its
//! [`supervise`] task started, which says each start's [`Outcome`] on
//! the state, or both let go; [`up`] waits for a kept tool's start;
//! [`settle`] is what a start does when the program declared nothing,
//! the agent detached from every dependency tool;
//! [`tool`](crate::containers::deploy::tool) is one run of a
//! dependency eager tool's container (a tool on record's is the one
//! every tool has); [`retire`] is what a delete does to a tool's
//! keeping and its run; [`boot`] is every attached eager tool brought
//! up at the daemon's start; [`stop_all`] is the daemon stopping, the
//! supervisors stilled before the runs are stopped so that none comes
//! back; and [`agent_deleted`], [`template_deleted`] and
//! [`tool_deleted`] are what the deletes — an agent's, an agent
//! template's, a dependency template's, a tool's — do to the tools
//! whose rows went with them and those they were attached to.
//!
//! Its own files are flattened into it, so everything is named
//! through this module and not through the file it lives in.

mod boot;
mod deleted;
mod held;
mod keep;
mod outcome;
mod retire;
mod settle;
mod state;
mod stop_all;
mod supervise;
mod up;

pub use boot::*;
pub use deleted::*;
pub use held::*;
use keep::*;
pub use outcome::*;
pub use retire::*;
pub use settle::*;
pub use state::*;
pub use stop_all::*;
pub use supervise::*;
pub use up::*;
