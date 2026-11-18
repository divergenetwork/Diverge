//! What `diverge update` takes.

use clap::Args;

/// The programs replaced by another version's.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct Update {
    /// The version to install, as `1.0.0`. Absent, the version next
    /// after the one this program is; where there is none newer, the
    /// command says so and changes nothing.
    #[arg(long, value_name = "VERSION")]
    pub version: Option<String>,
    /// The operating system to install for: `linux`, `macos` or
    /// `windows`. Absent, this host's.
    #[arg(long, value_name = "OS")]
    pub os: Option<String>,
    /// The architecture to install for: `x86_64` or `aarch64`.
    /// Absent, this host's.
    #[arg(long, value_name = "ARCH")]
    pub arch: Option<String>,
}
