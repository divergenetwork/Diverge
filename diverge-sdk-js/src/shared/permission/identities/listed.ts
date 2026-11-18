// The identities a grant lists.

import { z } from "zod";
import { Identity } from "../../identity.js";

/**
 * Exactly the identities listed, as a grant's `within` names them: the
 * object `{"identities":[…]}`, one member and no other, so that it
 * decodes as neither the word `"any"` nor a tags test. Each is one of the host's identities, by its issuer and its subject.
 * It reaches those and no other, whatever their tags; one that does
 * not exist, or no longer does, is reached by nothing. An empty list
 * reaches nothing.
 */
export const Listed = z.strictObject({
  /** The identities reached. */
  identities: z.array(Identity),
});

export type Listed = z.infer<typeof Listed>;
