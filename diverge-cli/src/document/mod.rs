//! The four documents in the root, as the CLI reads and writes them.
//!
//! The root holds one document per program — `daemon.yaml`,
//! `provider.yaml`, `viewer.yaml`, `cli.yaml` — and this module
//! handles all four the same way: a document is [`read`] as its
//! program reads it, the built-in defaults where there is no file,
//! and held as JSON, which the command that asked reads as the
//! document's own type — the daemon's `Config`, the provider's, the
//! viewer's, the CLI's — so that a file its program would refuse is
//! refused here; and a document is [`write()`]ten as that type's own
//! YAML, whole, in place. What is changed between the two is one
//! setting, by the command for it: no document is rewritten by a
//! program or a patch.
//!
//! # What a rewrite costs
//!
//! The file written is the type's serialization of the value: what
//! the document's program reads, exactly, and nothing else. A comment
//! in the file is gone after a write, and the keys come out in the
//! type's own order; the CLI's daemons and accounts keep the order
//! the document gave them.
//!
//! Its own files are flattened into it, so everything is named
//! through this module and not through the file it lives in.

mod error;
mod read;
mod which;
mod write;

pub use error::*;
pub use read::*;
pub use which::*;
pub use write::*;
