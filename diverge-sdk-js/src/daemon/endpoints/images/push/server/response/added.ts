// Platforms of the image, added.

import { z } from "zod";
import { Source } from "../../../../../../shared/containers/response/source.js";

/**
 * Some of the platforms a push named, now in the daemon's Store on the
 * provider: one of the pushes a push is made of, ended well. A push
 * whose source is the daemon is one push to the provider for each
 * source the daemon found, all at once, and each sends one of these as
 * it ends; a push whose source is the client sends one. What was added
 * stays, whatever the push ends as.
 */
export const Added = z.object({
  /** The platforms now held. */
  platforms: z.array(z.string()),
  /** The image's size in BYTES as the provider counts it once these are held; absent where the provider held them all already. */
  bytes: z.number().int().nonnegative().optional(),
  /** Where the provider took them from: its own Store, where it held them already; a registry it uses; or its caller, the daemon, which proved the source holds them. */
  source: Source,
});
export type Added = z.infer<typeof Added>;
