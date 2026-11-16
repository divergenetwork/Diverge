//! The room a filesystem has.

use std::path::Path;

use super::Error;

/// The bytes free to the caller on the filesystem `path`, a folder
/// that is there, is on: what a copy onto it is measured against
/// before it begins. Asked of `GetDiskFreeSpaceExW` on Windows.
#[cfg(windows)]
pub(super) async fn free(path: &Path) -> Result<u64, Error> {
    use std::os::windows::ffi::OsStrExt as _;

    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let mut free = 0u64;
    // SAFETY: `wide` is a NUL-terminated UTF-16 path that outlives the
    // call, `free` is a `u64` the call writes once, and the two other
    // out-pointers are null, which the call takes as "not asked".
    let done = unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), &mut free, std::ptr::null_mut(), std::ptr::null_mut()) };
    if done == 0 {
        return Err(Error::io(path, std::io::Error::last_os_error()));
    }
    Ok(free)
}

/// The bytes free to the caller on the filesystem `path`, a folder
/// that is there, is on: what a copy onto it is measured against
/// before it begins. Asked of `df -Pk`, whose second line's fourth
/// field is the kilobytes available, in the one form POSIX fixes.
#[cfg(not(windows))]
pub(super) async fn free(path: &Path) -> Result<u64, Error> {
    let mut command = tokio::process::Command::new("df");
    command.arg("-Pk").arg(path);
    let answer = crate::host::tools::capture("df", command).await?;
    answer
        .lines()
        .nth(1)
        .and_then(|line| line.split_whitespace().nth(3))
        .and_then(|kilobytes| kilobytes.parse::<u64>().ok())
        .map(|kilobytes| kilobytes * 1024)
        .ok_or_else(|| Error::Tool(crate::host::tools::Error::Output { program: "df".to_string() }))
}
