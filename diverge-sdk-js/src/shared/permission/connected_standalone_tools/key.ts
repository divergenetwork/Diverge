// One connected standalone tool, as a grant lists it.

import { z } from "zod";
import { DaemonId } from "../../../daemon/endpoints/daemons/id.js";
import { Tool } from "../../../daemon/reference/tool.js";

/**
 * One connected standalone tool a grant reaches, by what names it once
 * and for all on the daemon that holds it: the daemon the tool is on,
 * the tool as that daemon names it, and its number among every
 * connected tool ever held from that daemon's tool. No other member
 * decodes.
 */
export const Key = z.strictObject({
  /** The daemon the tool is on, by the ID of the holding daemon's record of it. */
  daemon: DaemonId,
  /** The tool, as that daemon names it and as the create named it. Compared, not read. */
  tool: Tool,
  /** The tool's number among all connected tools ever held from that daemon's tool. */
  index: z.number().int().nonnegative(),
});

export type Key = z.infer<typeof Key>;
