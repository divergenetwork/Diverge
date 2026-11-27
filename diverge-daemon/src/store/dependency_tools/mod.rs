//! The dependency tool records: a dependency template under one owner,
//! as its image's scope decides, with the agents attached to it.
//!
//! [`DependencyTool`] is one as the store holds it: the template it is
//! of, by id and whole, its scope and the owner the scope key names —
//! the one agent, or nobody — its tags, who
//! first made it and when. Beside the row are its [`attachments`] to
//! agents, in attach order, each noting when the agent's registration
//! last declared the template: every agent whose last registration did,
//! which is what holds an eager tool's container up. The container is work made from the row and is nowhere in it;
//! what runs is the live state's. [`all`], [`by_id`], [`by_scope`],
//! [`by_reference`], [`of_template`], [`of_owner_agent`] and
//! [`of_agent`] load; [`find_or_create`] makes one under its scope key,
//! or finds the one there is; [`delete_of_template`] and
//! [`delete_of_owner_agent`] remove, each answering the rows removed
//! so their states can be retired; [`set_tags`] writes the tags;
//! [`set_permissions`] keeps what its container last declared it may
//! ask of the daemon.
//!
//! Its own files are flattened into it, so everything is named through
//! this module and not through the file it lives in.

mod activity;
mod create;
mod delete;
mod load;
mod permissions;
mod row;
mod tags;
mod tool;

pub mod attachments;

pub use activity::*;
pub use create::*;
pub use delete::*;
pub use load::*;
pub use permissions::*;
pub use tags::*;
pub use tool::*;
