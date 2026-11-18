// The vault entries a grant lists.

import { z } from "zod";

/**
 * Exactly the vault entries listed, as a grant's `within` names them: the
 * object `{"keys":[…]}`, one member and no other, so that it
 * decodes as neither the word `"any"` nor a tags test. Each is a vault entry by its key.
 * It reaches those and no other, whatever their tags; one that does
 * not exist, or no longer does, is reached by nothing. An empty list
 * reaches nothing.
 */
export const Listed = z.strictObject({
  /** The vault entries reached. */
  keys: z.array(z.string()),
});

export type Listed = z.infer<typeof Listed>;
