// Publishing images to a registry as one image.

import { z } from "zod";
import { json, tagged } from "../../../../../../wire/encode.js";
import { Identity } from "../../../../../agent_log/identity.js";
import { Relay } from "../../../../../../shared/relay.js";
import { Reference } from "../../../../../../shared/containers/request/reference.js";
import { Description } from "../../../../../../shared/description.js";
import { Credential } from "./credential.js";
import { Part } from "./part.js";

/**
 * Ask the daemon to publish one or more images to a registry as one
 * image. The daemon picks a provider, brings onto it whatever it
 * lacks from wherever it finds it, merges the parts where there is
 * more than one or one is not whole, and has the provider send the
 * result to the registry.
 */
export const Frame = z.object({
  /** The images, and the platforms taken of each: one or more. One that takes every platform its manifest names is published as it is; anything else is merged into one new image first. A platform two of them take is `platform_repeated`. */
  images: z.array(Part).min(1),
  /** Where it goes: the registry, and the repository path there. The registry is one the provider the publish is done on uses. */
  reference: Reference,
  /** The tags to put on it there. Absent or empty, the image is published by its digest alone. */
  tags: z.array(z.string()).optional(),
  /** What to write to the registry as. Absent, the registry is asked as nobody. */
  credential: Credential.optional(),
  /** What an image this request makes in the Store is kept as. Absent, none. Not used where the image was in the Store already. */
  description: Description.optional(),
  /** The provider to do it on. Absent, the daemon picks one: of those the caller's grant reaches that keep a Store and use the registry, the one holding the most of the platforms taken already. */
  provider: Identity.optional(),
  /** Served by another daemon: see `Relay`. Absent, served here. */
  relay: Relay.optional(),
});
export type Frame = z.infer<typeof Frame>;

/** This frame's tag among the scope-opening requests: one byte at the front of the payload. */
const TAG = 164;

/** The payload: the tag, then the frame as JSON, as every request of the daemon's is. */
export function encode(frame: Frame): Uint8Array {
  return tagged(TAG, json(frame));
}
