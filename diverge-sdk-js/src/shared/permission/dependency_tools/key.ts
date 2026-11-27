// One dependency tool, as a grant lists it.

import { z } from "zod";
import { Key as Agent } from "../agents/key.js";

/**
 * Whose a listed dependency tool is: the one agent it is for, or
 * nobody in particular. Externally tagged:
 * `{"agent":{"template":…,"index":…}}`, or the string `"template"`. An
 * agent is named by its template and index and never by its name.
 */
export const Owner = z.union([
  z.strictObject({ agent: Agent }),
  z.literal("template"),
]);

export type Owner = z.infer<typeof Owner>;

/**
 * One dependency tool a grant reaches, by what names it once and for
 * all on the daemon that holds it: the dependency template it runs
 * from, and whose it is. No other member decodes.
 */
export const Key = z.strictObject({
  /** The dependency template, by id. */
  template: z.string(),
  /** Whose the tool is. */
  owner: Owner,
});

export type Key = z.infer<typeof Key>;
