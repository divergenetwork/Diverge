//! The daemon's state made whole at its start, after a daemon that
//! ended without stopping.
//!
//! A daemon that is stopped takes every run down and says so in its
//! log, frees what the run held, and only then lets the root go. One
//! that is killed does none of it. What it leaves in Postgres is
//! whole all the same, since every write of more than one row is one
//! transaction; what it leaves torn is outside Postgres, or is a row
//! that speaks of a run that lived in memory alone. [`run`] mends
//! all of that, once, before the daemon is built: before any provider
//! is dialled and before the port is listened on, so nothing is live
//! beside a log that is not whole. It is the same pass after a clean
//! stop, where it finds nothing to do.
//!
//! # The rules, in order
//!
//! 1. **A log folder with no record is removed** — see [`orphans`]. A
//!    delete commits its transaction and then removes the folder, so
//!    a death between the two leaves the folder.
//! 2. **Every log's files are made whole** — see
//!    [`logs::repair`](crate::logs::repair). A death under an append
//!    tears the end of `log` or of `index`.
//! 3. **Every run a log still holds deployed is closed** — see
//!    [`close`]. Its open asks are answered interrupted, `inactive`
//!    is kept where it was working, and `undeployed` is kept with the
//!    reason [`DaemonDied`](diverge_sdk::daemon::lifecycle::Reason::DaemonDied):
//!    what the run's own end would have kept, in its order.
//! 4. **The record of a run so closed is told it went down**: its
//!    `last_deployed`, and its `last_active` where it was working,
//!    are now.
//! 5. **Every vault slot is freed of its holder.** No run survives a
//!    restart, so no hold does.
//!
//! Rule 1 is done before the folders are walked; rules 2, 3 and 4 are
//! done folder by folder, in that order within one folder, many
//! folders at once; rule 5 beside them.
//!
//! # What it does not mend
//!
//! A volume's tags, where the provider that held the volume no longer
//! has it: which volumes there are is the providers' to say, and none
//! is dialled yet.
//!
//! # A failure
//!
//! The store refusing, or a log's file that cannot be read or cut, is
//! the pass's failure and the start's: the daemon does not run beside
//! a log it would append to blindly. The error names the file.
//!
//! Its own files are flattened into it, so everything is named
//! through this module and not through the file it lives in.

mod close;
mod error;
mod orphans;
mod run;

pub use close::*;
pub use error::*;
pub use orphans::*;
pub use run::*;
