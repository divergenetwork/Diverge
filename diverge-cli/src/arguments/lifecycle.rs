//! Start, or stop: what `diverge daemon` and `diverge provider` each
//! take beside `config`.

use clap::Subcommand;

/// The two things done to a program on this host: see
/// [`lifecycle`](crate::lifecycle) for each in full. The program is
/// the daemon's or the provider's own binary beside this one, run on
/// the CLI's root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum Lifecycle {
    /// Start the program on the root, detached; the command ends
    /// once the program is up, and fails if one already runs on the
    /// root or the program dies before it is up, with what it said.
    Start,
    /// Stop the program running on the root, asked over its own port
    /// on this host; the command ends once the whole of the
    /// program's teardown is done.
    Stop,
}
