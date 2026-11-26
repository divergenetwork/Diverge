// The compartments a grant lists.

import { z } from "zod";
import { Compartment } from "../../../daemon/endpoints/postgres/compartments/compartment.js";

/**
 * Exactly the compartments listed, as a grant's `within` names them:
 * the object `{"compartments":[…]}`, one member and no other, so that
 * it decodes as neither the word `"any"` nor a tags test. Each is a
 * `Compartment`, an agent in it by its template and index: one naming
 * an agent by name reaches nothing. An empty list reaches nothing.
 */
export const Listed = z.strictObject({
  /** The compartments reached. */
  compartments: z.array(Compartment),
});

export type Listed = z.infer<typeof Listed>;
