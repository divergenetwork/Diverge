//! Dependency tools: one per dependency template and owner, as far as
//! its image's scope shares it, with the agents that depend on it
//! and the tool running from it.
//!
//! A dependency template is a
//! [`Template`](crate::shared::containers::dependencies::Template) an
//! agent's program declares at its registration, of a dependency
//! tool's image — lazy, run while it is used, or eager, kept while
//! any agent is attached, as the image's `mode` says, under the scope
//! the image's `scope` names — held by its id, the hash of its
//! canonical bytes, so the same template declared twice is one. A
//! DEPENDENCY TOOL is that template under one OWNER, which the
//! image's scope decides: a template of an image whose scope is
//! `instance` is one tool per agent instance, owned by that agent;
//! one whose scope is `global` is one tool, owned by nobody.
//! Nothing on this wire makes one: the first registration that
//! declares the template under that owner does, and every registration
//! RESETS what the agent depends on — exactly the tools of the
//! templates it declared then, each noting when the agent last
//! declared it, and a tool it declared before and not again is its
//! dependency no more. A tool outlives the registrations. An instance
//! tool goes with its agent's delete, its `instance` compartment with
//! it; no request here deletes a tool, and what removes a template's
//! every tool, the global one among them, is the delete of the
//! dependency template's
//! [compartment](crate::daemon::endpoints::postgres::compartments::delete).
//! [`get`] answers one tool by its template and its owner, as a list
//! would report it; [`list`] lists them, narrowed, each with the
//! agents that depend on it — at most one for an instance tool, every
//! agent for a global one — and the one run from it, if its container
//! is up; [`tag`] and [`untag`] change one tool's tags, which are the
//! tool's and not in the template's hash.
//!
//! # The run
//!
//! A tool has at most one run: a lazy tool's container, while it is
//! used — for the `instance` scope, while its agent's container runs —
//! and an eager tool's, while the daemon keeps it up.
//! It is listed on the tool, by where it runs, and nowhere else —
//! never among the [`standalone_tools`](super::standalone_tools), whose
//! endpoints find no dependency tool. A tool is named by its template
//! and its owner, a
//! [`reference::Dependency`](crate::daemon::reference::Dependency),
//! for the four operations on a running container's files:
//! [`filetree`] watches one whole; [`download`] sends the client a
//! file or a directory out of it, [`upload`] puts files into it, and
//! [`transfer`] copies out of it into an agent, a tool, a dependency
//! tool or a volume, the bytes never reaching the client. Each finds
//! the tool running or answers `NotFound`, and starts and stops
//! nothing. [`logs`](super::logs) reads what the daemon kept of every
//! call and read it routed into one, running or not.

pub mod download;
pub mod exec;
pub mod filetree;
pub mod frontend;
pub mod get;
pub mod list;
pub mod tag;
pub mod terminal;
pub mod transfer;
pub mod untag;
pub mod upload;
