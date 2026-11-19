// Making a request of others.

import { z } from "zod";
import { json, tagged } from "../../../../../../wire/encode.js";
import { Relay } from "../../../../../../shared/relay.js";
import { To } from "./to.js";

/** Make a request of others, and wait for the one answer it gets. */
export const Frame = z.object({
  /** The request: any JSON value. The daemon reads none of it, and every listener is sent it as it is. */
  request: z.json(),
  /** Whom it is made of: see `To`. */
  to: To,
  /** Served by another daemon: see `Relay`. Absent, served here. */
  relay: Relay.optional(),
});
export type Frame = z.infer<typeof Frame>;

/** This frame's tag among the scope-opening requests: one byte at the front of the payload. */
const TAG = 166;

/** The payload: the tag, then the frame as JSON, as every request of the daemon's is. */
export function encode(frame: Frame): Uint8Array {
  return tagged(TAG, json(frame));
}
