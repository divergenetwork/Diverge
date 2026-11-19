// Changing the description of one image in a Store of the daemon's.

import { z } from "zod";
import { json, tagged } from "../../../../../../wire/encode.js";
import { Identity } from "../../../../../agent_log/identity.js";
import { Relay } from "../../../../../../shared/relay.js";
import { Description } from "../../../../../../shared/description.js";

/**
 * Ask the daemon to change the description of one image in its Store
 * on a provider: the provider and the digest name the image, as
 * `list` answers them. The daemon asks the provider as itself, so
 * what is edited is the daemon's own image there. The description is
 * the one thing of an image that changes.
 */
export const Frame = z.object({
  /** The provider whose Store of the daemon's holds the image. */
  provider: Identity,
  /** Which image, by its manifest digest, `<algorithm>:<hex>`. */
  digest: z.string(),
  /** The description the image has from now on. Absent, or `null`, the image is left with none. */
  description: Description.nullable().optional(),
  /** Served by another daemon: see `Relay`. Absent, served here. */
  relay: Relay.optional(),
});
export type Frame = z.infer<typeof Frame>;

/** This frame's tag among the scope-opening requests: one byte at the front of the payload. */
const TAG = 168;

/** The payload: the tag, then the frame as JSON, as every request of the daemon's is. */
export function encode(frame: Frame): Uint8Array {
  return tagged(TAG, json(frame));
}
