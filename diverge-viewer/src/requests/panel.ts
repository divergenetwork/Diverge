// The requests panel: a second card over the board, beside the
// outliner, listing every request made of us on every tab; and the
// sheet a request opens in, along the bottom, where it is answered.

import { getCurrentWindow, UserAttentionType } from "@tauri-apps/api/window";
import { keep, kept } from "../lib/keep";
import { ago } from "../sheet/ago";
import { contentElement } from "../sheet/media";
import { sender } from "../sheet/sender";
import { TABS } from "../tabs/tabs";
import { answer, arriving, type Asked, dismiss, heard, standing, watch } from "./hub";
import "./panel.css";

/** How the panel stands, as it is kept from one run to the next. */
type Stood = { open: boolean; left: number; top: number };

/** How long the panel flashes for a request that came in, in milliseconds. */
const FLASH = 1600;

/** No images are known here: a tool that asked is named by its record alone. */
const NO_IMAGES = new Map<string, string>();

/**
 * Start the requests panel.
 *
 * The rules:
 *
 * 1. A card over the board, under the tab strip and the sheets: a
 *    header — Requests, how many stand, fold — dragged by, and, open,
 *    the list. Folded, the header alone. Where it stands and whether
 *    it is open are kept (`keep`).
 * 2. The list is every request standing on every tab, newest first,
 *    scrolled when it is long: who made it, the tab it came in on,
 *    how long ago, and how it begins. ✕ puts one aside without
 *    answering it. Under the list, each tab whose listening does not
 *    stand says why, once.
 * 3. A request that comes in flashes the panel, and asks for the
 *    window's attention where the window is not the one in use.
 * 4. Clicking a request opens it in a sheet along the bottom: who made
 *    it and its tab, what it says, drawn as a log draws content, and a
 *    box to answer in. Enter sends; Shift+Enter breaks the line. Sent,
 *    the sheet says whether the answer was taken, and closes when it
 *    was. A request withdrawn while it is open closes its sheet, and
 *    says so.
 */
export function requests(): void {
  const was = kept<Partial<Stood>>("requests");
  let open = typeof was?.open === "boolean" ? was.open : true;
  let left = typeof was?.left === "number" ? was.left : 16;
  let top = typeof was?.top === "number" ? was.top : TABS + 16;
  let shown: string | null = null;

  // Rule 1.
  const root = document.createElement("aside");
  root.className = "requests";
  root.setAttribute("aria-label", "Requests");
  const head = document.createElement("div");
  head.className = "head";
  const title = document.createElement("span");
  title.className = "title";
  title.textContent = "Requests";
  const count = document.createElement("span");
  count.className = "count";
  const fold = document.createElement("button");
  fold.type = "button";
  fold.className = "fold";
  head.append(title, count, fold);
  const list = document.createElement("div");
  list.className = "list";
  const said = document.createElement("div");
  said.className = "said";
  root.append(head, list, said);
  document.body.append(root);

  const place = () => {
    left = Math.max(0, Math.min(window.innerWidth - root.offsetWidth, left));
    top = Math.max(TABS, Math.min(window.innerHeight - 40, top));
    Object.assign(root.style, { left: `${left}px`, top: `${top}px` });
    root.dataset.open = String(open);
    fold.textContent = open ? "–" : "+";
    fold.title = open ? "Fold" : "Unfold";
    keep("requests", () => ({ open, left, top }) satisfies Stood);
  };
  fold.addEventListener("click", () => {
    open = !open;
    place();
  });
  let grab: { x: number; y: number; left: number; top: number } | null = null;
  head.addEventListener("pointerdown", (event) => {
    if (event.target === fold) return;
    grab = { x: event.clientX, y: event.clientY, left, top };
    head.setPointerCapture(event.pointerId);
  });
  head.addEventListener("pointermove", (event) => {
    if (!grab) return;
    left = grab.left + event.clientX - grab.x;
    top = grab.top + event.clientY - grab.y;
    place();
  });
  head.addEventListener("pointerup", () => (grab = null));
  for (const type of ["pointerdown", "wheel"] as const) root.addEventListener(type, (event) => event.stopPropagation());
  window.addEventListener("resize", place);

  // Rule 4: the sheet.
  const sheet = document.createElement("section");
  sheet.className = "sheet request-sheet";
  sheet.hidden = true;
  const sheetHead = document.createElement("header");
  const sheetTitle = document.createElement("span");
  sheetTitle.className = "title";
  const sheetSub = document.createElement("span");
  sheetSub.className = "sub";
  const exit = document.createElement("button");
  exit.type = "button";
  exit.className = "exit";
  exit.textContent = "✕";
  exit.title = "Close";
  sheetHead.append(sheetTitle, sheetSub, exit);
  const thread = document.createElement("div");
  thread.className = "thread";
  const body = document.createElement("div");
  body.className = "body";
  const note = document.createElement("p");
  note.className = "note";
  const composer = document.createElement("div");
  composer.className = "composer";
  const row = document.createElement("div");
  row.className = "row";
  const text = document.createElement("textarea");
  text.rows = 1;
  text.placeholder = "Answer the request";
  const send = document.createElement("button");
  send.type = "button";
  send.className = "send";
  send.textContent = "Send";
  row.append(text, send);
  composer.append(row);
  thread.append(body, note, composer);
  sheet.append(sheetHead, thread);
  document.body.append(sheet);
  for (const type of ["keydown", "keyup"] as const) sheet.addEventListener(type, (event) => event.stopPropagation());

  const close = () => {
    shown = null;
    sheet.hidden = true;
    draw();
  };
  exit.addEventListener("click", close);
  const ready = () => {
    const request = standing().find((r) => r.key === shown);
    send.disabled = !request || request.answering || text.value.trim() === "";
  };
  const sent = async () => {
    const key = shown;
    if (key === null || send.disabled) return;
    const said = text.value;
    note.textContent = "sending…";
    ready();
    const outcome = await answer(key, said);
    if ("failed" in outcome) {
      note.textContent = `not sent: ${outcome.failed}`;
      ready();
      return;
    }
    text.value = "";
    if (outcome.taken) {
      close();
      return;
    }
    // Not taken: someone answered first, or it was withdrawn. The sheet stays to say so.
    note.textContent = "not delivered: the request was answered or withdrawn first";
    send.disabled = true;
  };
  text.addEventListener("input", ready);
  text.addEventListener("keydown", (event) => {
    if (event.key === "Enter" && !event.shiftKey && !event.isComposing) {
      event.preventDefault();
      void sent();
    }
  });
  send.addEventListener("click", () => void sent());

  const show = (request: Asked) => {
    shown = request.key;
    const from = sender(request.from, NO_IMAGES);
    sheetTitle.textContent = from.label;
    sheetSub.textContent = `request · ${request.label}`;
    body.replaceChildren(...request.request.map((block) => contentElement(block)));
    note.textContent = "";
    text.value = "";
    sheet.hidden = false;
    ready();
    text.focus();
    draw();
  };

  const draw = () => {
    const all = standing();
    count.textContent = all.length === 0 ? "" : String(all.length);
    root.classList.toggle("some", all.length > 0);
    // Rule 2.
    list.replaceChildren(
      ...all.map((request) => {
        const made = document.createElement("div");
        made.className = "row";
        made.classList.toggle("on", request.key === shown);
        const from = sender(request.from, NO_IMAGES);
        const who = document.createElement("span");
        who.className = `who ${from.tone}`;
        who.textContent = from.label;
        const where = document.createElement("span");
        where.className = "where";
        where.textContent = `${request.label} · ${ago(request.at)}`;
        const first = request.request.find((block) => block.type === "text");
        const begins = document.createElement("span");
        begins.className = "begins";
        begins.textContent = typeof first?.text === "string" ? first.text : "(no text)";
        const away = document.createElement("button");
        away.type = "button";
        away.className = "away";
        away.textContent = "✕";
        away.title = "Dismiss";
        away.addEventListener("click", (event) => {
          event.stopPropagation();
          dismiss(request.key);
        });
        made.append(who, where, away, begins);
        made.addEventListener("click", () => show(request));
        return made;
      }),
    );
    if (all.length === 0) {
      const none = document.createElement("p");
      none.className = "none";
      none.textContent = "No requests.";
      list.append(none);
    }
    const lines: string[] = [];
    for (const [tab, how] of heard()) {
      if (how.kind === "failed") lines.push(`${tab === "board" ? "Home" : tab}: ${how.why}`);
    }
    said.textContent = lines.join("\n");
    said.hidden = lines.length === 0;
    // Rule 4: the one open was withdrawn, or answered elsewhere.
    if (shown !== null && !all.some((r) => r.key === shown) && !sheet.hidden && note.textContent === "") {
      note.textContent = "this request was withdrawn";
      send.disabled = true;
    }
    ready();
  };

  // Rule 3.
  let flashing: number | undefined;
  arriving(() => {
    root.classList.add("flash");
    window.clearTimeout(flashing);
    flashing = window.setTimeout(() => root.classList.remove("flash"), FLASH);
    if (!document.hasFocus()) void getCurrentWindow().requestUserAttention(UserAttentionType.Informational).catch(() => undefined);
  });
  watch(draw);
  place();
  draw();
}
