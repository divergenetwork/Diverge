//! The grants a tool's program declares for itself at its
//! registration, and the actions every grant of the daemon's is made
//! of.
//!
//! A GRANT is one permission over one kind of thing the daemon holds
//! — agents, agent templates, standalone tools, their templates,
//! connected tools, dependency tools, providers, the daemons through
//! providers, daemons, accounts, roles, volumes, the vault, the
//! database, the events, images, the host's identities — and on the
//! wire it is one object with one member, named for the kind, whose
//! value is the kind's permission: `{"agents":…}`, `{"volumes":…}`.
//! The shapes are the daemon's own, stated in
//! [`daemon::grant`](crate::daemon::grant) — to make, over what
//! exists, tagging, the one thing.
//!
//! # A grant reaches all, exactly these, or by tags
//!
//! How far a grant over what exists reaches is its `within`, a
//! [`Reach`], and it is one of three and never two:
//!
//! 1. **All**: the string `"any"`, every one of the kind.
//! 2. **Exactly these**: the kind's own list — its `Listed`, an object
//!    of one member — naming the records one by one, each by what
//!    names it once and for all: `{"ids":[…]}`, `{"volumes":[…]}`,
//!    `{"agents":[…]}`. It reaches those and no other, whatever their
//!    tags.
//! 3. **By tags**: a [`Tags`] test, those carrying every one of some
//!    tags and any one of others.
//!
//! A list and tags are never in one grant: there is no writing it. A
//! tool that is to reach these records and also everything tagged so
//! declares two grants, since grants only add and none takes from
//! another. A tagging grant's `within` is the same three; its `tags`,
//! which tags it may touch, is `"any"` or a listed set. Over agents
//! alone there are two words more, [`agents::Within`]: `"attached"`,
//! the agents attached to the tool, and `"never_active"`, the agents
//! that have never run.
//!
//! Where the daemon's own `within` is the kind's list filter, which
//! may narrow a reach further — by creator, by template, by what is
//! running — a grant here is the reach and nothing more.
//!
//! # A list is the arguments', never the image's
//!
//! A tool's image is meant to travel — run by one caller and another,
//! declaring the same everywhere — and a record is one daemon's: an
//! id, an index, a name there. So an image declares no list of its own
//! choosing. A program whose ARGUMENTS name records — the agents it
//! is to message, the daemon it is to relay to, the volume it is to
//! read — reads them out of the arguments it was registered with and
//! declares those, so the same image declares one list here and
//! another there, and is held to whichever it was given. A tag is a
//! convention a template may assume, and `"any"` and the two words
//! name nobody, so those an image may declare as constants.
//!
//! The database and the events, of each of which there is one, are a
//! bare array of actions each, as they are for the daemon. The grants
//! over providers, through them and over images list providers and
//! read the tags of the providers' records.
//!
//! # The actions are defined once
//!
//! Each kind's [`Make`](agents::Make) and [`Over`](agents::Over) — what
//! brings one into being, what is done to one that exists — live here,
//! and the daemon's grants are built of them:
//! [`daemon::grant`](crate::daemon::grant) re-exports every one, so a
//! grant of a template and a grant of a role hold the same actions and
//! differ only in how far they reach. [`Tagging`] and [`Within`] are
//! the daemon's too, defined here for the same reason; [`Reach`] is
//! this module's alone.
//!
//! # Where it is carried
//!
//! A tool's program answers its registration with a list of
//! [`Grant`]s — the `permissions` of the
//! [`register`](crate::container_proxy::inside::register) answer — at
//! every start of its container, a standalone tool's and a dependency
//! tool's alike. The proxy and the provider relay them and read
//! nothing; the daemon holds the run to exactly these and judges
//! everything the tool asks of it by them, as
//! [`daemon::grant`](crate::daemon::grant) states a request is judged.
//! Every type here derives a JSON Schema, because an image embeds its
//! declarations in the schema of its own documentation.

mod grant;
mod reach;
mod tags;
mod tagging;
mod within;

pub use grant::*;
pub use reach::*;
pub use tags::*;
pub use tagging::*;
pub use within::*;

pub mod accounts;
pub mod agents;
pub mod agents_templates;
pub mod connected_standalone_tools;
pub mod daemons;
pub mod dependency_tools;
pub mod events;
pub mod identities;
pub mod images;
pub mod postgres;
pub mod postgres_compartments;
pub mod providers;
pub mod providers_daemons;
pub mod roles;
pub mod standalone_tools;
pub mod standalone_tools_templates;
pub mod vault;
pub mod volumes;
