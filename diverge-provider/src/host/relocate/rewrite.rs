//! A path's place changed in the files that name it.

use std::path::Path;

use super::Error;

/// A path as a JSON document podman wrote holds it: the text between
/// the quotes. Podman's encoder writes a backslash doubled and `<`,
/// `>` and `&` as `<`, `>` and `&`, and leaves every
/// other character of a path as it is; so does this. A separator the
/// path ends in is left off, so that the text is a prefix of every
/// path under it.
pub(super) fn written(path: &Path) -> Result<String, Error> {
    let text = path.to_str().ok_or_else(|| Error::NotText(path.to_path_buf()))?;
    let text = text.trim_end_matches(['/', '\\']);
    let quoted = serde_json::to_string(text).map_err(|_| Error::NotText(path.to_path_buf()))?;
    let inner = &quoted[1..quoted.len() - 1];
    Ok(inner.replace('<', "\\u003c").replace('>', "\\u003e").replace('&', "\\u0026"))
}

/// Where in `text` a path under one of `olds` begins, and how long the
/// old prefix found there is: the first such place at or after `from`.
///
/// A place counts only where the prefix is the start of a JSON string
/// — a quote before it — and is a whole path or a whole folder of one
/// — a quote, a slash or a doubled backslash after it. So a path that
/// merely begins with the same letters, `…/data2`, is not taken for
/// one under `…/data`, and nothing inside a longer string is touched.
/// On Windows a path is compared without regard to the case of its
/// ASCII letters, as the filesystem compares it.
#[cfg(not(target_os = "linux"))]
fn find(text: &str, lowered: &str, olds: &[String], from: usize) -> Option<(usize, usize)> {
    let hay = if cfg!(windows) { lowered } else { text };
    let mut first: Option<(usize, usize)> = None;
    for old in olds {
        let needle = if cfg!(windows) { old.to_ascii_lowercase() } else { old.clone() };
        if needle.is_empty() {
            continue;
        }
        let mut at = from;
        while let Some(found) = hay[at..].find(&needle) {
            let start = at + found;
            let end = start + needle.len();
            let before = text[..start].ends_with('"');
            let after = &text[end..];
            let whole = after.starts_with('"') || after.starts_with('/') || after.starts_with("\\\\");
            if before && whole {
                if first.is_none_or(|(place, _)| start < place) {
                    first = Some((start, needle.len()));
                }
                break;
            }
            // One character on: a path begins with a slash, a drive
            // letter or a backslash, but nothing here assumes it.
            at = start + needle.chars().next().map_or(1, char::len_utf8);
        }
    }
    first
}

/// Whether `text` names a path under one of `olds`.
#[cfg(not(target_os = "linux"))]
pub(super) fn names(text: &str, olds: &[String]) -> bool {
    find(text, &text.to_ascii_lowercase(), olds, 0).is_some()
}

/// `text` with every path under one of `olds` put under `new`
/// instead, each of them a path as [`written`]; `None` where it names
/// none, and is left as it is. Nothing but the prefix changes: the
/// rest of the document is kept byte for byte, whatever it holds.
#[cfg(not(target_os = "linux"))]
pub(super) fn replace(text: &str, olds: &[String], new: &str) -> Option<String> {
    let lowered = text.to_ascii_lowercase();
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while let Some((start, length)) = find(text, &lowered, olds, at) {
        out.push_str(&text[at..start]);
        out.push_str(new);
        at = start + length;
    }
    if at == 0 {
        return None;
    }
    out.push_str(&text[at..]);
    Some(out)
}
