// The image, as the registry holds it.

import { z } from "zod";
import { Identity } from "../../../../../agent_log/identity.js";
import { Reference } from "../../../../../../shared/containers/request/reference.js";

/** The image published: the registry holds it under the repository named, with every tag asked for. */
export const Published = z.object({
  /** The provider it was done on, and in whose Store of the daemon's the image is: the one the request named, or the one the daemon picked. */
  provider: Identity,
  /** The manifest digest the registry holds it by: a part's own digest where one part was published as it is, the merged image's otherwise. */
  digest: z.string(),
  /** Whether this request made the image in the daemon's Store on the provider. `true`: it was not there before. `false`: it was there already. */
  created: z.boolean(),
  /** Where it went, as the request named it. */
  reference: Reference,
  /** The tags put on it there, as the request named them. */
  tags: z.array(z.string()),
});
export type Published = z.infer<typeof Published>;
