// The one image, of the parts.

import { z } from "zod";

/** The one image the parts of a publish make, in the daemon's Store on the provider. */
export const Merged = z.object({
  /** Its manifest digest, `<algorithm>:<hex>`: what is published, and the same for the same parts on every provider. */
  digest: z.string(),
  /** Whether this request made it: `false` where the daemon owned an image of the digest there already, and nothing was added. */
  created: z.boolean(),
  /** Its size in BYTES as the provider counts it; absent where it was there already. */
  bytes: z.number().int().nonnegative().optional(),
});
export type Merged = z.infer<typeof Merged>;
