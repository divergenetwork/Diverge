// One compartment, as a list reports it.

import { z } from "zod";
import type { Keyed } from "../../../../../../../shared/listing/keyed.js";
import { Compartment, Kind } from "../../../compartment.js";

/**
 * One compartment as a list and a get report it: which it is, what
 * kind of container it is of, its tags, since when it is on record,
 * whether a delete would be allowed now, and how many connections
 * containers hold through it now.
 */
export const Listed = z.object({
  /** Which compartment, an agent in it by its template and index. */
  compartment: Compartment,
  /** What kind of container it is of. An `image` compartment's is the `type` of the image's document. */
  kind: Kind,
  /** Its tags, sorted bytewise. Absent when empty. */
  tags: z.array(z.string()).optional(),
  /** Since when it is on record: an RFC 3339 timestamp in UTC. */
  created: z.iso.datetime({ offset: true }),
  /** Whether a delete would be allowed now, connections apart. */
  deletable: z.boolean(),
  /** How many connections containers hold through it now. While it is not `0` a delete answers `in_use`. */
  connections: z.number().int().nonnegative(),
});
export type Listed = z.infer<typeof Listed>;

/** Keyed by the compartment itself. */
export const key: Keyed<Listed> = (listed) => listed.compartment;
