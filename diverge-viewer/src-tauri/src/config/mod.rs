//! The one `viewer.yaml` Diverge Viewer reads.
//!
//! - `--config <dir>`, else `DIVERGE_CONFIG`, else `~/.diverge/`,
//!   names the ROOT, created if absent — see [`root`]. The root is
//!   one directory for every Diverge program on the host, each with
//!   its own document in it. The app keeps no files of its own under
//!   it.
//! - `<root>/viewer.yaml` is read if present; absent means the
//!   built-in default, [`Config::default`], which names no daemon —
//!   and an app with no daemon does not start. No other name or
//!   extension is looked for — see [`load`].
//! - [`Config`] is the document; [`Daemon`] its `daemon`, which says
//!   which daemon the app dials and as which of the root's
//!   identities. A key the document does not have is refused, so a
//!   misspelled setting is refused rather than ignored.
//!
//! ```yaml
//! daemon:
//!   address: ws://127.0.0.1:14980/
//!   issuer: https://auth.example.com
//!   subject: alice
//!   redial: 5     # seconds, or `never`; absent, 5
//! ```
//!
//! [`Error`] is why the root or the file could not be used.
//!
//! The document is the app's own type, kept here as the daemon and the
//! provider each keep theirs in a `config` folder of their own. The
//! CLI reads the same document with a type of its own crate,
//! `diverge-viewer/config`, since it cannot take one from a Tauri
//! app: the two types state one document, and a change to either's
//! fields is a change to both.
//!
//! Its own files are flattened into it, so everything is named
//! through this module and not through the file it lives in.

mod config;
mod daemon;
mod error;
mod load;
mod root;

pub use config::*;
pub use daemon::*;
pub use error::*;
pub use load::*;
pub use root::*;
