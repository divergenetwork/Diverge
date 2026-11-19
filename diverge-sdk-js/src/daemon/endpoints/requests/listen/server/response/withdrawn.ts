// A request wanted no more.

import { z } from "zod";
import { RequestId } from "../../../id.js";

/**
 * One request that is no longer to be answered: another listener
 * answered it, or its creator took it back, or went. An id the
 * listener was never sent is to be ignored.
 */
export const Withdrawn = z.object({
  /** Which request. */
  id: RequestId,
});
export type Withdrawn = z.infer<typeof Withdrawn>;
