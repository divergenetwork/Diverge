//! What an image may call itself, and what a repository may be named.

/// The most bytes an image's name holds, and a repository's.
pub const NAME_BYTES: usize = 128;

/// Whether `name` may be an image's name. It is what one path
/// component of an OCI repository name may be, but that its letters
/// may be capitals. The rules:
///
/// 1. It is 1 to [`NAME_BYTES`] bytes long.
/// 2. It is runs of `a` to `z`, `A` to `Z` and `0` to `9`, joined by
///    separators. No other byte is in it: no space, nothing that is
///    not ASCII.
/// 3. A separator is `.`, `_`, `__`, or one or more `-`.
/// 4. It neither begins nor ends with a separator.
///
/// [`Document::parse`](super::Document::parse) holds every document's
/// `name` to them. Case is kept and is told apart: `Agent` and `agent`
/// are two names.
pub fn named(name: &str) -> bool {
    if name.is_empty() || name.len() > NAME_BYTES {
        return false;
    }
    let alnum = |byte: u8| byte.is_ascii_alphanumeric();
    let bytes = name.as_bytes();
    if !alnum(bytes[0]) || !alnum(bytes[bytes.len() - 1]) {
        return false;
    }
    let mut index = 0;
    while index < bytes.len() {
        if alnum(bytes[index]) {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && !alnum(bytes[index]) {
            index += 1;
        }
        let separator = &bytes[start..index];
        if !(separator == b"." || separator == b"_" || separator == b"__" || separator.iter().all(|byte| *byte == b'-')) {
            return false;
        }
    }
    true
}

/// Whether `name` may be a registry's namespace or repository: one
/// path component of an OCI repository name. It is an image's name
/// by every rule of [`named`], and one more:
///
/// 5. It has no capital: its letters are `a` to `z` alone, as OCI
///    requires of a repository, and as the programs that push and
///    pull one require.
///
/// So the two differ in case and in nothing else, and an image's name
/// in lowercase is always a name a repository may have.
pub fn named_lowercase(name: &str) -> bool {
    named(name) && !name.bytes().any(|byte| byte.is_ascii_uppercase())
}
