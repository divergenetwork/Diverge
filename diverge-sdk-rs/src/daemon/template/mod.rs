//! A template: what an agent, or a tool, is made from, held by its
//! hash.
//!
//! A template is everything about an agent or a tool that is not its
//! name, not the provider it runs on, not its mounts, and not what
//! its image says of itself: the image, what a container of it is
//! given beyond what the image needs, and the arguments — so that a template is
//! shareable, the same template on any daemon hashing the same. What
//! the image says of itself, in its
//! [document](crate::shared::containers::info::Document), no template
//! says again: the memory and the disk it needs, and how a tool is kept running. The shape is one, [`Template`], written once
//! here; which of the two it is for is its `type`, the first member,
//! `"agent"` for an [agent
//! template](crate::daemon::endpoints::agents::templates) and `"tool"`
//! for a [tool template](crate::daemon::endpoints::standalone_tools::templates),
//! so the two kinds never hash the same and a template's text says what
//! it is for. Each family makes, lists and deletes its own. Every
//! volume a container mounts is not a template's but a create's.
//!
//! # The id is the template's hash
//!
//! The lowercase hexadecimal SHA-256 of the template's
//! [`canonical`](crate::shared::canonical) bytes — its compact JSON,
//! absent members omitted, the image's `references` absent too, since
//! one image under different names is one template, no whitespace,
//! every object key sorted at every depth, the `arguments` and every
//! map inside them included; see
//! [`Template::hashed`]. Sixty-four
//! characters. The daemon computes it on a create and
//! answers it; a caller may compute it the same way and need not.
//!
//! # The arguments are shown only where asked, and allowed
//!
//! A get and a list answer a template as a [`Shown`]: the template,
//! its `arguments` present only where the request said
//! `"arguments": true`, which takes the `arguments` action over the
//! template beside the `get` or the `list`.

mod shown;
mod template;

pub use shown::*;
pub use template::*;
