// One standalone tool, as a grant lists it.

import { z } from "zod";

/**
 * One standalone tool a grant reaches, by what names it once and for
 * all on the daemon that holds it: the template it was made from and
 * its number among every tool that daemon ever made from that
 * template. No name is carried, and no other member decodes.
 */
export const Key = z.strictObject({
  /** The template the tool was made from, by id. */
  template: z.string(),
  /** The tool's number among all tools the daemon ever made from that template. */
  index: z.number().int().nonnegative(),
});

export type Key = z.infer<typeof Key>;
