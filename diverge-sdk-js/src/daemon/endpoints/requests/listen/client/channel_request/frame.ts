// The channels a caller opens on a listen: an answer, and the cancel.

import { json, tagged } from "../../../../../../wire/encode.js";
import type { Respond } from "./respond.js";

/**
 * What a client asks the daemon for while a listen is open.
 *
 * - `cancel`, tag `0`, carrying nothing: stop listening. Nothing comes
 *   back on this channel; what comes back is the scope's finish.
 * - `respond`, tag `1`, then the `Respond` as JSON: answer a request.
 *   Answered with exactly one delivery — whether the answer was taken
 *   — then the finish.
 */
export type Frame = "cancel" | { respond: Respond };

/** Tag for `cancel`. */
const CANCEL = 0;

/** Tag for `respond`. */
const RESPOND = 1;

/** A tag, then the answer's JSON; the cancel is the tag alone. */
export function encode(frame: Frame): Uint8Array {
  return frame === "cancel" ? Uint8Array.of(CANCEL) : tagged(RESPOND, json(frame.respond));
}
