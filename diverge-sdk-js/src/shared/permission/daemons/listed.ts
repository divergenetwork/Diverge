// The remote daemons a grant lists.

import { z } from "zod";
import { DaemonId } from "../../../daemon/endpoints/daemons/id.js";

/**
 * Exactly the remote daemons listed, as a grant's `within` names them: the
 * object `{"ids":[…]}`, one member and no other, so that it
 * decodes as neither the word `"any"` nor a tags test. Each is a remote daemon by the ID of the daemon's record of it.
 * It reaches those and no other, whatever their tags; one that does
 * not exist, or no longer does, is reached by nothing. An empty list
 * reaches nothing.
 */
export const Listed = z.strictObject({
  /** The remote daemons reached. */
  ids: z.array(DaemonId),
});

export type Listed = z.infer<typeof Listed>;
