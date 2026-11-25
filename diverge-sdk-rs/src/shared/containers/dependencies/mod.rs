//! The dependencies a container declares, asked of the caller: each a
//! tool container, as a template the caller deploys unread.
//!
//! An AGENT container's program answers its registration with the tools
//! it depends on — each a [`Template`]: a tool container the caller
//! runs, by its image — a digest, and where its bytes may be fetched —
//! what it is given beyond what the image needs, its arguments, and
//! the agent's files and directories the caller serves live into a
//! lazy tool of one agent — and named by its id, the hash of its
//! canonical bytes, which the caller serves it to the program under.
//! What kind of tool it is, is not the declaration's to say: the
//! image is a dependency tool's, and its own
//! [document](crate::shared::containers::info::Document) says how it
//! is kept running, how far it is shared, the secrets it needs and
//! the memory and the disk it needs. The proxy carries the list on the agents
//! begin's `Begun`, beside the secrets the program declared; the
//! provider carries both to the caller in one
//! [`registration`](super::registration) ask on the run scope, and the
//! caller deploys each before it answers. The run goes on to its id
//! on a deploy and ends, the container stopped, on anything else. An
//! agent that declares nothing — no dependency and no secret — is
//! never asked about.
//!
//! A TOOL container has no dependencies: its program may answer its
//! registration with a list, and its proxy ignores it; its begin's
//! `Begun` carries its secrets' declaration alone, and its
//! registration ask carries an empty list. Only an agent has
//! dependencies. A dependency's own container is a tool container,
//! registered like any other, its secrets resolved through the
//! binding of the agent that spawned it.
//!
//! The provider carries and does not read the list; everything a caller
//! needs to deploy a dependency is in it, as fields a program acts on
//! and not as words a person reads — which is the difference between a
//! dependency tool template and the tool templates a daemon keeps for
//! itself, and why one travels inside an image.

mod mount;
mod template;

pub use mount::*;
pub use template::*;
