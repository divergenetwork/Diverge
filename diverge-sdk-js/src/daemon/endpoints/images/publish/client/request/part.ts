// One image a publish takes platforms from.

import { z } from "zod";

/** One image of a publish, by its digest, and the platforms taken of it. */
export const Part = z.object({
  /** Which image, by its manifest digest, `<algorithm>:<hex>`. */
  digest: z.string(),
  /** The platforms to take of it: one or more, each once, each a platform string such as `linux/amd64`. Absent, every platform its manifest names. */
  platforms: z.array(z.string()).min(1).optional(),
});
export type Part = z.infer<typeof Part>;
