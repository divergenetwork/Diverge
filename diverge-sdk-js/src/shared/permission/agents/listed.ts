// The agents a grant lists.

import { z } from "zod";
import { Key } from "./key.js";

/**
 * Exactly the agents listed, as a grant's `within` names them: the
 * object `{"agents":[…]}`, one member and no other, so that it
 * decodes as neither the word `"any"` nor a tags test. Each is a `Key`: the template and index that are one agent's.
 * It reaches those and no other, whatever their tags; one that does
 * not exist, or no longer does, is reached by nothing. An empty list
 * reaches nothing.
 */
export const Listed = z.strictObject({
  /** The agents reached. */
  agents: z.array(Key),
});

export type Listed = z.infer<typeof Listed>;
