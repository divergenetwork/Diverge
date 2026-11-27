// Whose a dependency tool is: one agent's, or everyone's.

import { z } from "zod";
import { Agent } from "./agent.js";

/**
 * Whose a dependency tool is, as its template's `scope` named it:
 * the one agent, once and for all; or every agent that declares it.
 * Externally tagged: `{"agent":{"template":…,"index":…}}`, or the
 * string `"template"`.
 */
export const Owner = z.union([
  z.strictObject({ agent: Agent }),
  z.literal("template"),
]);
export type Owner = z.infer<typeof Owner>;
