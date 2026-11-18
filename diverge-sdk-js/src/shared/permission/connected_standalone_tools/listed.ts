// The connected standalone tools a grant lists.

import { z } from "zod";
import { Key } from "./key.js";

/**
 * Exactly the connected standalone tools listed, as a grant's `within` names them: the
 * object `{"tools":[…]}`, one member and no other, so that it
 * decodes as neither the word `"any"` nor a tags test. Each is a `Key`: the daemon the tool is on, the tool there, and its index.
 * It reaches those and no other, whatever their tags; one that does
 * not exist, or no longer does, is reached by nothing. An empty list
 * reaches nothing.
 */
export const Listed = z.strictObject({
  /** The connected standalone tools reached. */
  tools: z.array(Key),
});

export type Listed = z.infer<typeof Listed>;
