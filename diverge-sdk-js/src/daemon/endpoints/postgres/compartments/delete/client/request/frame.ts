// Deleting a compartment.

import { z } from "zod";
import { json, tagged } from "../../../../../../../wire/encode.js";
import { Relay } from "../../../../../../../shared/relay.js";
import { Compartment } from "../../../compartment.js";

/**
 * Ask the daemon to delete a compartment: its role, its schema and
 * every table in it. Only a compartment whose list item says
 * `deletable` is deleted.
 */
export const Frame = z.object({
  /** Which compartment. An agent is named by its name or by its template and index. */
  compartment: Compartment,
  /** Served by another daemon: see `Relay`. Absent, served here. */
  relay: Relay.optional(),
});
export type Frame = z.infer<typeof Frame>;

/** This frame's tag among the scope-opening requests: one byte at the front of the payload. */
const TAG = 172;

/** The payload: the tag, then the frame as JSON, as every request of the daemon's is. */
export function encode(frame: Frame): Uint8Array {
  return tagged(TAG, json(frame));
}
