// Whose a dependency tool is: one agent's, or everyone's.

import { z } from "zod";
import { Agent } from "./agent.js";

/**
 * Whose a dependency tool is, as its template's `scope` names it:
 * the one agent it is for, named either way; or nobody in
 * particular. Externally tagged: `{"agent":…}`, or the string
 * `"template"`. The daemon answers the once-and-for-all
 * form, as `key.Owner`.
 */
export const Owner = z.union([
  z.strictObject({ agent: Agent }),
  z.literal("template"),
]);
export type Owner = z.infer<typeof Owner>;
