// Deleting one image in a Store of the daemon's, or some of its platforms.

import { z } from "zod";
import { json, tagged } from "../../../../../../wire/encode.js";
import { Identity } from "../../../../../agent_log/identity.js";
import { Relay } from "../../../../../../shared/relay.js";

/**
 * Ask the daemon to delete one image in its Store on a provider, or
 * some of its platforms: the provider and the digest name the image,
 * as `list` answers them. The daemon asks the provider as itself, so
 * what is deleted is the daemon's own image there.
 */
export const Frame = z.object({
  /** The provider whose Store of the daemon's the image is deleted from. */
  provider: Identity,
  /** The image's manifest digest, `<algorithm>:<hex>`. */
  digest: z.string(),
  /**
   * The platforms to delete of it: each once, each a platform string
   * such as `linux/amd64`. Absent or empty, the whole image. Every one
   * must be a platform the daemon holds of the image there, or the
   * delete is `platform_not_held` and nothing is deleted. Naming every
   * platform held deletes the image. The digest is the image still
   * where some stay.
   */
  platforms: z.array(z.string()).optional(),
  /** Served by another daemon: see `Relay`. Absent, served here. */
  relay: Relay.optional(),
});
export type Frame = z.infer<typeof Frame>;

/** This frame's tag among the scope-opening requests: one byte at the front of the payload. */
const TAG = 163;

/** The payload: the tag, then the frame as JSON, as every request of the daemon's is. */
export function encode(frame: Frame): Uint8Array {
  return tagged(TAG, json(frame));
}
