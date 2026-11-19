//! The theme, served to every frontend at `/diverge/`: the shared
//! `theme.css`, and the `theme.js` that keeps a page's `data-theme` the
//! app's, so an agent's or a tool's own page can wear the person's theme.
//!
//! The rules:
//!
//! 1. Two paths on every frontend's origin are the app's own, never sent
//!    on to the program: `/diverge/theme.css` and `/diverge/theme.js`,
//!    each the file the repository holds (`diverge-theme/`), built into
//!    the app. A program serves only paths under `/frontend`, so none of
//!    its own is ever hidden.
//! 2. Each is answered to a `GET` or a `HEAD`, with its type and
//!    `no-cache`, so a page always takes the app's current file; any
//!    other method is `405`.
//! 3. A page takes them by choice: one that does not ask for them is
//!    never given them. The page is its maker's; the frame round it is
//!    the app's.

use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};

/// Rule 1: the shared theme, as the repository holds it.
const CSS: &str = include_str!("../../../diverge-theme/theme.css");

/// Rule 1: the page's side of the theme.
const JS: &str = include_str!("../../../diverge-theme/theme.js");

/// Rules 1 and 2: the app's own answer to a frontend's request for
/// `path`, or `None` when the path is the program's.
pub fn theme(method: &Method, path: &str) -> Option<Response> {
    let (kind, body) = match path {
        "/diverge/theme.css" => ("text/css; charset=utf-8", CSS),
        "/diverge/theme.js" => ("text/javascript; charset=utf-8", JS),
        _ => return None,
    };
    if method != Method::GET && method != Method::HEAD {
        return Some((StatusCode::METHOD_NOT_ALLOWED, [(header::ALLOW, "GET, HEAD")]).into_response());
    }
    Some((StatusCode::OK, [(header::CONTENT_TYPE, kind), (header::CACHE_CONTROL, "no-cache")], body).into_response())
}
