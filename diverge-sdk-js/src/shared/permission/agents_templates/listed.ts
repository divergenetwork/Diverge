// The agent templates a grant lists.

import { z } from "zod";

/**
 * Exactly the agent templates listed, as a grant's `within` names them: the
 * object `{"ids":[…]}`, one member and no other, so that it
 * decodes as neither the word `"any"` nor a tags test. Each is a template by its id.
 * It reaches those and no other, whatever their tags; one that does
 * not exist, or no longer does, is reached by nothing. An empty list
 * reaches nothing.
 */
export const Listed = z.strictObject({
  /** The agent templates reached. */
  ids: z.array(z.string()),
});

export type Listed = z.infer<typeof Listed>;
