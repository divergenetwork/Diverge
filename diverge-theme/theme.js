// The theme, kept by a frontend the viewer shows: an agent's or a
// tool's own page, loaded beside `/diverge/theme.css`, asks the app which
// theme the person has on and keeps its own `<html data-theme>` to it.
//
// The rules:
//
// 1. The viewer serves this file and theme.css at `/diverge/` on every
//    frontend's own origin (`src-tauri/src/theme.rs`); a page takes them
//    with `<link rel="stylesheet" href="/diverge/theme.css">` and
//    `<script src="/diverge/theme.js"></script>`, after its own rules, and
//    names colours, sizes and spaces only by the theme's roles. Anywhere
//    else both are missing and the page keeps its own.
// 2. Loaded inside a frame, it asks the window holding it once:
//    `{ diverge: "theme?" }`.
// 3. An answer, then and whenever the person changes theme, is
//    `{ diverge: "theme", theme }`: a theme's id sets `data-theme`, and
//    null or anything else clears it, which is the default, plum and gold.
//    Only the window holding the frame is heard.
(() => {
  const root = document.documentElement;
  const holder = window.parent;
  // Rule 1: not inside a frame, nothing to ask.
  if (holder === window) return;
  window.addEventListener("message", (event) => {
    // Rule 3.
    if (event.source !== holder) return;
    const said = event.data;
    if (!said || typeof said !== "object" || said.diverge !== "theme") return;
    if (typeof said.theme === "string" && /^[a-z0-9-]+$/.test(said.theme)) root.dataset.theme = said.theme;
    else delete root.dataset.theme;
  });
  // Rule 2.
  holder.postMessage({ diverge: "theme?" }, "*");
})();
