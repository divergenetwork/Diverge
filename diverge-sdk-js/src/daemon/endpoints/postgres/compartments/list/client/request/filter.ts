// What narrows a list of compartments.

import { z } from "zod";
import { Compartment, Database, Kind } from "../../../compartment.js";

/**
 * The filter over compartments: every member optional, and every one
 * given a condition a compartment passes or does not. In a list
 * request it is flattened into the request and narrows what the
 * daemon sends; in a grant it is read as a test. A filter with no
 * member given passes everything. A member that lists candidates
 * matches any one of them; an empty list is absent, and matches
 * everything.
 */
export const Filter = z.object({
  /** Exactly these, each as the daemon answers it: an agent by its template and index, never by name. */
  compartments: z.array(Compartment).optional(),
  /** Any one of these of a container's three: `"instance"`, `"template"`, `"image"`. */
  databases: z.array(Database).optional(),
  /** Of any one of these kinds of container. */
  kinds: z.array(Kind).optional(),
  /** Of any one of these templates, by id: a `template` compartment's own, an agent's or a tool's `instance`'s, a dependency tool's `instance`'s dependency template. */
  templates: z.array(z.string()).optional(),
  /** Of any one of these images, by digest: an `image` compartment's own. */
  images: z.array(z.string()).optional(),
  /** Whether a delete would be allowed now, connections apart — `true` — or not, `false`; absent, either. */
  deletable: z.boolean().optional(),
  /** Every one of these among the thing's tags. Absent when empty, and then any tags. */
  all_tags: z.array(z.string()).optional(),
  /** Any one of these among the thing's tags; with `all_tags`, both hold. Absent when empty. */
  any_tags: z.array(z.string()).optional(),
  /** The earliest `created` to list, inclusive; absent, no earliest. */
  created_from: z.iso.datetime({ offset: true }).optional(),
  /** The latest `created` to list, inclusive; absent, no latest. */
  created_to: z.iso.datetime({ offset: true }).optional(),
});
export type Filter = z.infer<typeof Filter>;
