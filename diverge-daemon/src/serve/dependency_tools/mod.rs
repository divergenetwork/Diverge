//! The requests over dependency tools, served: [`list`], [`get`],
//! [`tag`] and [`untag`] over the rows — one per template and owner —
//! and
//! [`filetree`], [`download`], [`upload`] and [`transfer`] over one's
//! container, named by its template and whose it is, and the two
//! through its program's doors, [`frontend`] and [`api`], which are
//! one handler told which door it is. One handler
//! per request, each the shape [`serve`](crate::serve) states. What a
//! reference names is [`resolve`]d to the row and its run, if it has
//! one; a request into a container deploys a tool that is down. What a tool
//! is attached to and whether it runs — the
//! [`Facts`](crate::judge::filter::dependency_tools::Facts) a grant
//! and a filter judge by — are [`load`]ed as a [`Loaded`], and whether
//! the caller's grants [`reaches`] a tool is one test.

mod door;
mod facts;
mod resolve;

pub use facts::*;
pub use resolve::*;

pub mod api;
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
