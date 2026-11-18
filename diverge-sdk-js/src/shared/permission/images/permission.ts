// One grant's worth over images.

import { z } from "zod";
import { reach } from "../reach.js";
import { Action } from "./action.js";
import { Listed } from "./listed.js";

/**
 * What one grant over images allows, in the one shape `permission`
 * states for actions over what exists: an object of `Action`s and how
 * far they reach, by the tags of the providers whose Stores they act
 * on. An object that is not this does not decode, and neither does a
 * bare array of actions.
 */
export const Permission = z.strictObject({
  /** The actions. */
  actions: z.array(Action),
  /** How far: `"any"`, the providers listed, or the tags as a test. */
  within: reach(Listed),
});
export type Permission = z.infer<typeof Permission>;
