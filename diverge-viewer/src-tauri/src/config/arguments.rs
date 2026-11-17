//! What the app is started with.

use std::ffi::OsString;
use std::path::PathBuf;

use super::Error;

/// The arguments after the program's name: where the root is, and
/// which daemon is dialled as which account.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Arguments {
    /// `--config <dir>`: the root. Absent, see [`root`](super::root).
    pub config: Option<PathBuf>,
    /// `--daemon <name>`: the daemon to dial, by its name in the
    /// settings. Absent, the default daemon.
    pub daemon: Option<String>,
    /// `--account <name>`: the account to dial it as, by its name
    /// under the daemon. Absent, the daemon's default account.
    pub account: Option<String>,
}

/// `args` — the arguments after the program's name — read. Each of
/// `--config <dir>`, `--daemon <name>` and `--account <name>` may be
/// given once, in any order. Any other argument, one given twice, or
/// one with nothing after it, is [`Error::Arguments`].
pub fn arguments(args: impl Iterator<Item = OsString>) -> Result<Arguments, Error> {
    let mut read = Arguments::default();
    let mut args = args;
    while let Some(arg) = args.next() {
        let said = arg.to_string_lossy().into_owned();
        let taken = match said.as_str() {
            "--config" => read.config.is_some(),
            "--daemon" => read.daemon.is_some(),
            "--account" => read.account.is_some(),
            _ => true,
        };
        let (false, Some(value)) = (taken, args.next()) else {
            return Err(Error::Arguments(said));
        };
        match said.as_str() {
            "--config" => read.config = Some(PathBuf::from(value)),
            "--daemon" => read.daemon = Some(value.to_string_lossy().into_owned()),
            _ => read.account = Some(value.to_string_lossy().into_owned()),
        }
    }
    Ok(read)
}
