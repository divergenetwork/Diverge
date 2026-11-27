// One request a requester put to the daemon.

import { z } from "zod";

/** One request: its path, its body (a template create's `arguments` replaced), and whether the daemon let it through. */
export const Command = z.object({
  type: z.literal("command"),
  /** The request's path, as the daemon names it: `agents/message`, `logs`. */
  path: z.string(),
  /** The request's body as sent, but that the `arguments` of a template made by `agents/templates/create` or `standalone_tools/templates/create` are replaced. */
  request: z.json(),
  /** Whether the daemon let it through: false exactly when it answered `Forbidden`. */
  allowed: z.boolean(),
});
export type Command = z.infer<typeof Command>;
