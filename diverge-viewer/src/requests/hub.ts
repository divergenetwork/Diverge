// The requests made of us, on every daemon a tab shows: listened for,
// kept while they stand, and answered.

import { daemon } from "@diverge/sdk";
import { handle, relayed, type Route } from "../lib/wire";

/** A content block, as a request and an answer carry them. */
export type Block = { type: string } & Record<string, unknown>;

/** Who made a request, as the daemon names a party. */
export type From = daemon.creator.Creator;

/** One request standing: which tab it came in on, who made it, what it says, and when it came. */
export type Asked = {
  /** The tab's key and the request's id: unique across tabs. */
  key: string;
  id: string;
  tab: string;
  /** The tab's name, as the strip shows it. */
  label: string;
  from: From;
  request: Block[];
  at: Date;
  /** An answer is on its way. */
  answering: boolean;
};

/** How a tab's listening stands: listening; another listener has it; or why it is not. */
export type Hearing = { kind: "listening" } | { kind: "elsewhere" } | { kind: "failed"; why: string };

/** How long after a listen ends the next is opened, in milliseconds: the pause the daemon's own dials take. */
const AGAIN = 5000;

type Listener = { label: string; route: Route; stopped: boolean; responder: { respond(id: string, response: Block[]): Promise<string>; cancel(): Promise<void> } | null };

const listeners = new Map<string, Listener>();
const asked = new Map<string, Asked>();
const hearing = new Map<string, Hearing>();
const told = new Set<() => void>();
const came = new Set<(request: Asked) => void>();

const say = () => {
  for (const heard of told) heard();
};

/** Every request standing, newest first. */
export const standing = (): Asked[] => [...asked.values()].sort((a, b) => b.at.getTime() - a.at.getTime());

/** How each open tab's listening stands, by the tab's key. */
export const heard = (): ReadonlyMap<string, Hearing> => hearing;

/** Be told whenever the requests standing, or how a tab's listening stands, change; returns how to stop. */
export function watch(changed: () => void): () => void {
  told.add(changed);
  return () => told.delete(changed);
}

/** Be told each request as it comes in; returns how to stop. */
export function arriving(arrived: (request: Asked) => void): () => void {
  came.add(arrived);
  return () => came.delete(arrived);
}

/** Every request of `tab` gone from the list. */
const drop = (tab: string) => {
  for (const [key, request] of asked) if (request.tab === tab) asked.delete(key);
};

/**
 * Listen for the requests made of us on the daemon `route` reaches, for
 * the tab `tab`, named `label`.
 *
 * The rules:
 *
 * 1. One listen for each tab open, on the daemon it shows: Home on our
 *    own, a daemon's tab through its route.
 * 2. Each request heard is kept and told (`arriving`); each withdrawn
 *    is dropped, whether or not it was kept.
 * 3. A daemon lets one party listen once. A listen answered
 *    `already_open` is another listener's — a tab that reaches the same
 *    daemon as the same account, another viewer, the command line —
 *    and is said so (`elsewhere`) and not an error: its requests are
 *    shown by whoever has it, where that is another tab of ours.
 * 4. A listen that ends — refused, or its connection gone — is opened
 *    again `AGAIN` later, for as long as the tab is open. What it had
 *    heard is dropped when it ends: the daemon tells it again what
 *    still stands.
 * 5. `unlisten` cancels the listen and drops what it heard.
 */
export function listen(tab: string, label: string, route: Route): void {
  if (listeners.has(tab)) return;
  const listener: Listener = { label, route, stopped: false, responder: null };
  listeners.set(tab, listener);
  void (async () => {
    while (!listener.stopped) {
      let why = "the listen ended";
      try {
        const [stream, responder] = await daemon.endpoints.requests.listen.client.execute(handle, { ...relayed(route) });
        listener.responder = responder;
        if (listener.stopped) {
          await responder.cancel();
          return;
        }
        hearing.set(tab, { kind: "listening" });
        say();
        for await (const frame of stream) {
          if (listener.stopped) return;
          if (frame === "already_open") {
            // Rule 3.
            hearing.set(tab, { kind: "elsewhere" });
            why = "";
          } else if (frame === "forbidden") {
            why = "this account may not listen here";
          } else if ("error" in frame) {
            why = "the daemon could not open the listen";
          } else if ("request" in frame) {
            // Rule 2.
            const key = `${tab} ${frame.request.id}`;
            const request: Asked = { key, id: frame.request.id, tab, label, from: frame.request.from, request: frame.request.request as Block[], at: new Date(), answering: false };
            asked.set(key, request);
            say();
            for (const arrived of came) arrived(request);
          } else {
            asked.delete(`${tab} ${frame.withdrawn.id}`);
            say();
          }
        }
      } catch (error) {
        why = error instanceof Error ? error.message : String(error);
      }
      if (listener.stopped) return;
      // Rule 4.
      listener.responder = null;
      drop(tab);
      if (why !== "") hearing.set(tab, { kind: "failed", why });
      say();
      await new Promise((resolve) => setTimeout(resolve, AGAIN));
    }
  })();
}

/** Rule 5 of `listen`. */
export function unlisten(tab: string): void {
  const listener = listeners.get(tab);
  if (!listener) return;
  listener.stopped = true;
  void listener.responder?.cancel().catch(() => undefined);
  listeners.delete(tab);
  hearing.delete(tab);
  drop(tab);
  say();
}

/** A request put aside: gone from the list, and not answered. The daemon still holds it for whoever else it was made of. */
export function dismiss(key: string): void {
  if (asked.delete(key)) say();
}

/**
 * Answer the request `key` with `text`: whether the answer was taken.
 *
 * Taken, the request is ours and is dropped. Not taken — another
 * answered first, or it was withdrawn — it is dropped all the same:
 * there is nothing left to answer. An answer that could not be put
 * leaves the request standing, and says why.
 */
export async function answer(key: string, text: string): Promise<{ taken: boolean } | { failed: string }> {
  const request = asked.get(key);
  const responder = request ? listeners.get(request.tab)?.responder : null;
  if (!request || !responder) return { failed: "the request is no longer standing" };
  request.answering = true;
  say();
  try {
    const delivery = await responder.respond(request.id, [{ type: "text", text }]);
    asked.delete(key);
    say();
    return { taken: delivery === "delivered" };
  } catch (error) {
    request.answering = false;
    say();
    return { failed: error instanceof Error ? error.message : String(error) };
  }
}
