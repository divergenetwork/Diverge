//! What `diverge update` takes.

use std::path::PathBuf;

use clap::Args;

/// The programs replaced by another version's, and the release's
/// skills installed where `SKILLS_DIR` says — or not at all, said so
/// with `--skip-updating-skills`. One of the two is required, so that
/// an update never leaves old skills beside new programs without
/// having been told to.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct Update {
    /// The directory the release's skills are installed in: each skill
    /// is a folder there, by its name. A folder of a skill's name
    /// already there is replaced whole; anything else in the directory
    /// is left as it is. Made if it is not there.
    #[arg(value_name = "SKILLS_DIR", required_unless_present = "skip_updating_skills", conflicts_with = "skip_updating_skills")]
    pub skills: Option<PathBuf>,
    /// Install no skills: the programs alone are replaced.
    #[arg(long)]
    pub skip_updating_skills: bool,
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
