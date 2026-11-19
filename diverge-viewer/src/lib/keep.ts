// What the viewer remembers from one run to the next: where nodes
// were put, what the outliner shows, and how the outliner stands. Kept
// in the page's own storage.

/** Every name kept is under this, so nothing else in the storage is touched. */
const PREFIX = "diverge-viewer:";

/**
 * What the Home board is kept under: the ID of the daemon the viewer
 * was started on, as the account it was started as, which the app
 * computes and hands the page before it runs. A viewer started on
 * another daemon, or as another account, has another. `home` where
 * the app handed none.
 */
export const HOME_ID: string = (globalThis as { __DIVERGE_HOME__?: string }).__DIVERGE_HOME__ ?? "home";

/** How long after the last change a value is written, in milliseconds. */
const WAIT = 400;

/** One value waiting to be written: how to read it now, and the timer that will. */
type Pending = { value: () => unknown; timer: number };

const pending = new Map<string, Pending>();

/** The text each name was last written as, so a value that has not changed is not written again. */
const written = new Map<string, string>();

/**
 * What was kept under `name`, or `null`.
 *
 * Storage that is not there, that refuses, or that holds something
 * that is not JSON is nothing kept: the viewer starts as it would with
 * none.
 */
export function kept<T>(name: string): T | null {
  try {
    const text = localStorage.getItem(PREFIX + name);
    if (text === null) return null;
    written.set(name, text);
    return JSON.parse(text) as T;
  } catch {
    return null;
  }
}

/** Write `name` now, if it waits, and if what it is now is not what was last written. */
function write(name: string): void {
  const waiting = pending.get(name);
  if (!waiting) return;
  window.clearTimeout(waiting.timer);
  pending.delete(name);
  try {
    const text = JSON.stringify(waiting.value());
    if (text === written.get(name)) return;
    localStorage.setItem(PREFIX + name, text);
    written.set(name, text);
  } catch {
    // Storage that refuses keeps nothing, and the viewer goes on.
  }
}

/**
 * Keep `value()` under `name`.
 *
 * The rules:
 *
 * 1. Nothing is written at the call. The value is read and written
 *    `WAIT` after the LAST call for its name, so a burst of changes —
 *    a drag, a resize — is one write, of what it ended as.
 * 2. A value that is, as text, what was last written or read under
 *    its name is not written.
 * 3. A page that is going — closed, reloaded — writes at once
 *    everything that waits, so the last change before it is kept.
 * 4. Storage that is not there, or is full, keeps nothing: no error
 *    reaches the caller.
 */
export function keep(name: string, value: () => unknown): void {
  const waiting = pending.get(name);
  if (waiting) window.clearTimeout(waiting.timer);
  pending.set(name, { value, timer: window.setTimeout(() => write(name), WAIT) });
}

// Rule 3.
window.addEventListener("pagehide", () => {
  for (const name of [...pending.keys()]) write(name);
});
