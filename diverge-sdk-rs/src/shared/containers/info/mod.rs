//! The image's own word on what its arguments may be: `/.diverge/DIVERGE.md`,
//! read of the image and answered on the image's own scope.
//!
//! Every container image ships one file in the folder `.diverge` at
//! the root of its filesystem, `/.diverge/DIVERGE.md` — the folder is
//! where an image keeps what it says of itself, and this file is the
//! one thing of it read: a frontmatter between two `---` lines
//! that is one JSON object — `type`, which is `agent`,
//! `standalone_tool` or `dependency_tool` and says which of the three
//! documents this is; `name`, what the image calls itself, which is
//! what a repository may be named but that it may have capitals
//! ([`named`]);
//! `description`, what it is, in one line (a
//! [`Description`](crate::shared::description::Description));
//! `schema`, the JSON Schema of the
//! [`arguments`](crate::shared::containers::request::Container::arguments)
//! the image's program accepts; `memory` and `disk`, what a container
//! of the image needs, in bytes; all six required; `secrets`, every
//! secret the program could take, each by its name and what it is
//! ([`Secret`]), where it takes any; `frontend`
//! where the program serves one; in an agent's and in no tool's,
//! `chat` and `structured_decisions`, each `false` or what the agent
//! says of itself ([`Chat`], [`StructuredDecisions`]), at least one of
//! them said; in a tool's of either kind and in no agent's, `mode`,
//! how its tools are kept running ([`Mode`]); in a dependency tool's
//! alone, `scope`, how far its tools are shared ([`Scope`]); and no
//! member besides — and then a markdown body: what the image runs.
//! The description and the body are the short and the long account of
//! the image, and both are the image's own: no party keeps another.
//! [`Document`] is that file parsed, and [`Document::parse`] states
//! the rules; [`Document::render`] writes one back, so that one shape
//! is written and read. The image's icons are files beside this one,
//! by fixed names, which the document does not mention: see
//! [`icon`](crate::shared::containers::icon). No file of the folder is
//! read past [`MAX_BYTES`]: neither this one nor an icon.
//!
//! The exchange that makes an image usable without knowing it: on an
//! [`images::read`](crate::provider::endpoints::images::read) scope
//! that holds the image, the caller opens a channel that carries
//! nothing — a direction with nothing to say has no request here, as
//! everywhere in this crate — and the provider answers with one
//! [`response::Frame`]: the document, parsed; that the image ships
//! none; or an error. Then the finish. It is asked of the image and of
//! nothing else: no container scope carries it, and no container is
//! run for it — the provider reads the file out of the image. The
//! program is never asked — the file is the image's, not the
//! program's — and an image that ships none is answered as missing,
//! which is a fact about the image and not a failure of anything.
//!
//! The schema is the ARGUMENTS', not the request's. The rest of the
//! request is this crate's to type; the arguments are the image's to
//! define, and the document is how the image says what it takes. A
//! schema constrains a document, which is exactly what that value is:
//! one value, sent once. What it cannot say — how a loop's chunks are
//! ordered, what a tool server's tools do — is not its job, and this
//! crate says that in prose as it does everywhere.

mod agent;
mod chat;
mod dependency_tool;
mod document;
mod false_or;
mod limit;
mod modalities;
mod modality;
mod mode;
mod name;
mod parse_error;
mod scope;
mod secret;
mod standalone_tool;
mod structured_decisions;

pub mod response;

pub use agent::*;
pub use chat::*;
pub use dependency_tool::*;
pub use document::*;
pub use false_or::*;
pub use limit::*;
pub use modalities::*;
pub use modality::*;
pub use mode::*;
pub use name::*;
pub use parse_error::*;
pub use scope::*;
pub use secret::*;
pub use standalone_tool::*;
pub use structured_decisions::*;
