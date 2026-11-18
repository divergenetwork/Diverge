// The roles a grant lists.

import { z } from "zod";

/**
 * Exactly the roles listed, as a grant's `within` names them: the
 * object `{"names":[…]}`, one member and no other, so that it
 * decodes as neither the word `"any"` nor a tags test. Each is a role by its name.
 * It reaches those and no other, whatever their tags; one that does
 * not exist, or no longer does, is reached by nothing. An empty list
 * reaches nothing.
 */
export const Listed = z.strictObject({
  /** The roles reached. */
  names: z.array(z.string()),
});

export type Listed = z.infer<typeof Listed>;
