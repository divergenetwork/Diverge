// The volumes a grant lists.

import { z } from "zod";
import { Key } from "./key.js";

/**
 * Exactly the volumes listed, as a grant's `within` names them: the
 * object `{"volumes":[…]}`, one member and no other, so that it
 * decodes as neither the word `"any"` nor a tags test. Each is a `Key`: the provider that holds the volume, and its name there.
 * It reaches those and no other, whatever their tags; one that does
 * not exist, or no longer does, is reached by nothing. An empty list
 * reaches nothing.
 */
export const Listed = z.strictObject({
  /** The volumes reached. */
  volumes: z.array(Key),
});

export type Listed = z.infer<typeof Listed>;
