//! The one `viewer.yaml` Diverge Viewer reads.
//!
//! - `--config <dir>`, else `DIVERGE_CONFIG`, else `~/.diverge/`,
//!   names the ROOT, created if absent — see [`root`]. The root is
//!   one directory for every Diverge program on the host, each with
//!   its own document in it. The app keeps no files of its own under
//!   it.
//! - `<root>/viewer.yaml` is read if present; absent means the
//!   built-in default, [`Config::default`]. No other name or
//!   extension is looked for — see [`load`].
//! - [`Config`] is the document: the `port` a stop is heard on, the
//!   `redial`, the `daemons` by name — each a [`Daemon`]: its
//!   `address`, its own `identity` where the app is to hold it to
//!   one, its `accounts` by name, each an [`Account`], and its
//!   `default` account — and the `default` daemon. They are the
//!   CLI's own `daemons` and `default`, member for member. A key the
//!   document does not have is refused.
//! - [`arguments`] reads what the app was started with, and
//!   [`select`] picks the daemon and the account a start is: the ones
//!   it names, else the defaults.
//!
//! ```yaml
//! port: 14981
//! redial: 5     # seconds, or `never`; absent, 5
//! default: local
//! daemons:
//!   local:
//!     address: ws://127.0.0.1:14980/
//!     default: me
//!     accounts:
//!       me:
//!         issuer: https://auth.example.com
//!         subject: alice
//! ```
//!
//! [`Error`] is why the arguments, the root or the file could not be
//! used, or a name not found.
//!
//! The document is the app's own type, kept here as the daemon and the
//! provider each keep theirs in a `config` folder of their own. The
//! CLI reads the same document with a type of its own
//! (`diverge-cli/src/config/viewer.rs`), since it cannot take one
//! from a Tauri app: the two types state one document, and a change to either's
//! fields is a change to both.
//!
//! Its own files are flattened into it, so everything is named
//! through this module and not through the file it lives in.

mod arguments;
mod config;
mod daemon;
mod error;
mod load;
mod root;
mod select;

pub use arguments::*;
pub use config::*;
pub use daemon::*;
pub use error::*;
pub use load::*;
pub use root::*;
pub use select::*;
