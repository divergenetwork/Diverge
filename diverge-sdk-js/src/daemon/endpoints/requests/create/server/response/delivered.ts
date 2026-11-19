// A listener the request was sent to.

import { z } from "zod";
import { Creator } from "../../../../../creator/creator.js";

/**
 * One listener was sent the request, just now: one that was listening
 * when the request was created, or one that came to listen after.
 * Sent once for each listener, and never again for it.
 */
export const Delivered = z.object({
  /** Who was sent it: an account, by its identity, or a tool, as the daemon names a party. */
  to: Creator,
});
export type Delivered = z.infer<typeof Delivered>;
