// The tool's run, while its container is up.

import { z } from "zod";
import { Provider } from "../../../../../agent_log/provider.js";

/**
 * The tool's container, while it runs: where, and under what id. A tool
 * has at most one run — for as long as anything holds it, an eager
 * tool's attachments among what does — and none while its
 * container is down: a tool between runs has no run until it
 * runs again.
 */
export const Run = z.object({
  /** The provider the container runs on. See `Provider`. */
  provider: Provider,
  /** The container's id, as the provider's run answered it. */
  id: z.string(),
});
export type Run = z.infer<typeof Run>;
