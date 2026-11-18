// The providers a grant lists.

import { z } from "zod";
import { Identity } from "../../identity.js";

/**
 * Exactly the providers listed, as a grant's `within` names them: the
 * object `{"identities":[…]}`, one member and no other, so that it
 * decodes as neither the word `"any"` nor a tags test. Each is a provider by its identity: the grant acts on the Stores of the providers listed.
 * It reaches those and no other, whatever their tags; one that does
 * not exist, or no longer does, is reached by nothing. An empty list
 * reaches nothing.
 */
export const Listed = z.strictObject({
  /** The providers reached. */
  identities: z.array(Identity),
});

export type Listed = z.infer<typeof Listed>;
