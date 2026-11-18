// One volume, as a grant lists it.

import { z } from "zod";
import { Identity } from "../../identity.js";

/**
 * One volume a grant reaches: the provider that holds it, by its
 * identity, and the name that provider lists it under. The pair is
 * the whole of how a volume is named. No other member decodes.
 */
export const Key = z.strictObject({
  /** The provider, by its identity. */
  provider: Identity,
  /** The name the provider lists the volume under; compared and not read. */
  name: z.string(),
});

export type Key = z.infer<typeof Key>;
