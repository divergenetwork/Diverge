// Merging images into one, in the daemon's Store on a provider.

import { z } from "zod";
import { json, tagged } from "../../../../../../wire/encode.js";
import { Identity } from "../../../../../agent_log/identity.js";
import { Relay } from "../../../../../../shared/relay.js";
import { Description } from "../../../../../../shared/description.js";
import { Part } from "../../../publish/client/request/part.js";

/**
 * Ask the daemon to make one image of one or more, in its Store on a
 * provider. The daemon brings onto that provider whatever of them its
 * Store there lacks, from wherever it finds it — every provider it is
 * connected to and every remote daemon on record — and has the
 * provider merge them.
 */
export const Frame = z.object({
  /** The provider the image is made on, and whose Store of the daemon's it goes into. */
  provider: Identity,
  /** The images, and the platforms taken of each: one or more. A platform two of them take is `platform_repeated`. One image that takes every platform it has is itself, and nothing is merged. */
  images: z.array(Part).min(1),
  /** What the image made is kept as in the Store. Required. Not written over an image of the same digest owned there already. */
  description: Description,
  /** Served by another daemon: see `Relay`. Absent, served here. */
  relay: Relay.optional(),
});
export type Frame = z.infer<typeof Frame>;

/** This frame's tag among the scope-opening requests: one byte at the front of the payload. */
const TAG = 167;

/** The payload: the tag, then the frame as JSON, as every request of the daemon's is. */
export function encode(frame: Frame): Uint8Array {
  return tagged(TAG, json(frame));
}
