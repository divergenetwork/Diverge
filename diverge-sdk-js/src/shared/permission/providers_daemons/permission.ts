// One grant's worth through providers.

import { z } from "zod";
import { reach } from "../reach.js";
import { Over } from "./over.js";
import { Listed } from "./listed.js";

/**
 * What one grant through providers allows, in the one shape
 * `permission` states for actions over what exists: an object of
 * `Over` actions and how far they reach, by the providers' tags. An
 * object that is not this does not decode.
 */
export const Permission = z.strictObject({
  /** The actions. */
  actions: z.array(Over),
  /** How far: `"any"`, the providers listed, or the tags as a test. */
  within: reach(Listed),
});
export type Permission = z.infer<typeof Permission>;
