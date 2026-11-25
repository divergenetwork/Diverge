// What tells a tool template from an agent template.

import { z } from "zod";

/**
 * `"type":"tool"`, and nothing beside it: what tells a tool template
 * from an agent template. Flat among the template's members. How
 * every tool made from the template is kept running is not here: it
 * is the image's own to say, the `mode` of its document.
 */
export const ToolType = z.object({
  type: z.literal("tool"),
});
export type ToolType = z.infer<typeof ToolType>;
