//! `diverge update`: the programs replaced by another version's.
//!
//! A release is one archive per platform, holding every Diverge
//! program, flat. An update downloads one and puts its programs
//! where the installed ones are, `bin` beneath the root.
//!
//! The rules:
//!
//! 1. **Which version.** The one asked for; else the version next
//!    after this program's own in the releases' list — one version
//!    up, and never further in one run. Where this program's version
//!    is the newest, nothing is done and the command says so; where
//!    it is not in the list at all, the newest is taken. See
//!    [`releases`].
//! 2. **Which platform.** The operating system and the architecture
//!    asked for, else this host's.
//! 3. **Nothing running.** The daemon's, the provider's and the
//!    viewer's locks on the root are taken first and held to the end:
//!    one that is held is that program running, and the update is
//!    refused before anything is downloaded.
//! 4. **Everything beside `bin` first.** The archive is downloaded
//!    into `temp/<version>` beneath the root — a folder of that name
//!    left by an earlier run deleted first — and unpacked there. An
//!    update stopped before the replacing begins has changed nothing
//!    in `bin`, and the next run deletes what it left.
//! 5. **Whole, or not begun.** The replacing begins only once every
//!    program a release holds is unpacked. See [`unpack`].
//! 6. **Replaced by renaming, the CLI last.** Each installed program
//!    is renamed aside and the new one renamed into its place, within
//!    the one root and so on the one disk; an interrupt is held off
//!    for the moment that takes. A rename that fails undoes the ones
//!    before it. The CLI itself is last: an update that ends part way
//!    leaves the CLI that began it, which can be run again. See
//!    [`swap`].
//! 7. **One line.** An update that succeeded says what it installed;
//!    one with nothing to do says that.
//!
//! Its own files are flattened into it, so everything is named
//! through this module and not through the file it lives in.

mod error;
mod releases;
mod skills;
mod swap;
mod unpack;
mod update;

pub use error::*;
pub use releases::*;
pub use skills::*;
pub use swap::*;
pub use unpack::*;
pub use update::*;
