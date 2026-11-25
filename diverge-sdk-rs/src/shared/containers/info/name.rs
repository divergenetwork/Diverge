//! What an image may call itself.

/// The most bytes an image's name holds.
pub const NAME_BYTES: usize = 128;

/// Whether `name` may be an image's name, which is what one path
/// component of an OCI repository name may be. An image's name and a
/// registry's namespace and repository are held to the one rule, by
/// this one function, so that an image's name is always a name a
/// repository may have. The rules:
///
/// 1. It is 1 to [`NAME_BYTES`] bytes long.
/// 2. It is runs of `a` to `z` and `0` to `9`, joined by separators.
///    No other byte is in it: no capital, no space, nothing that is
///    not ASCII.
/// 3. A separator is `.`, `_`, `__`, or one or more `-`.
/// 4. It neither begins nor ends with a separator.
///
/// [`Document::parse`](super::Document::parse) holds every document's
/// `name` to them.
pub fn named(name: &str) -> bool {
    if name.is_empty() || name.len() > NAME_BYTES {
        return false;
    }
    let alnum = |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit();
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
