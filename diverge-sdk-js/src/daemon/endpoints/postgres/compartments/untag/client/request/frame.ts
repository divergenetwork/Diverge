// Taking tags off a compartment.

import { z } from "zod";
import { json, tagged } from "../../../../../../../wire/encode.js";
import { Relay } from "../../../../../../../shared/relay.js";
import { Compartment } from "../../../compartment.js";

/**
 * Ask the daemon to take tags off a compartment. A tag not held is off it still, and
 * is not a failure.
 */
export const Frame = z.object({
  /** Which compartment. An agent is named by its name or by its template and index. */
  compartment: Compartment,
  /** The tags, one by one. Empty changes nothing and is not a failure. */
  tags: z.array(z.string()),
  /** Served by another daemon: see `Relay`. Absent, served here. */
  relay: Relay.optional(),
});
export type Frame = z.infer<typeof Frame>;

/** This frame's tag among the scope-opening requests: one byte at the front of the payload. */
const TAG = 171;

/** The payload: the tag, then the frame as JSON, as every request of the daemon's is. */
export function encode(frame: Frame): Uint8Array {
  return tagged(TAG, json(frame));
}
