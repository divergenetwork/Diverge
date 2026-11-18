// A blob the registry took.

import { z } from "zod";

/** One blob the provider sent to the registry: the publish's progress. Never sent for a blob the registry held already. */
export const Uploaded = z.object({
  /** The blob's digest, `<algorithm>:<hex>`. */
  digest: z.string(),
  /** Its size in BYTES. */
  bytes: z.number().int().nonnegative(),
});
export type Uploaded = z.infer<typeof Uploaded>;
