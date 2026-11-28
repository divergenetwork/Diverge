// A run's container up on a provider.

import { z } from "zod";
import { Cause } from "./cause.js";
import { Provider } from "./provider.js";

/**
 * The run's container came up on the provider named: the first item of
 * every run, before anything the run did. The provider's members lie
 * beside the item's own, and so does what deployed it.
 */
export const Deployed = z.object({
  ...Provider.shape,
  /** What deployed the run: the one thing that found its container down and started it. See `Cause`. */
  cause: Cause,
});
export type Deployed = z.infer<typeof Deployed>;
