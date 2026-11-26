// One grant's worth over the compartments.

import { z } from "zod";
import { reach } from "../reach.js";
import { Tagging } from "../tagging.js";
import { within } from "../within.js";
import { Over } from "./over.js";
import { Listed } from "./listed.js";

/**
 * What one grant over the compartments allows, in one of two shapes
 * told apart on the wire by their very form: an object of `Over`
 * actions and how far they reach; or an object of tagging actions,
 * how far they reach, and which tags. An object that is neither does
 * not decode. The shapes are tried in this order, as serde tries
 * them.
 */
export const Permission = z.union([
  /** Over compartments: the `Over` actions held, reaching every one, those listed, or only those the tags let through. */
  z.strictObject({
    /** The actions. */
    actions: z.array(Over),
    /** How far: `"any"`, the compartments listed, or the tags as a test. */
    within: reach(Listed),
  }),
  /** Tagging compartments: putting tags on, taking them off, or both. */
  z.strictObject({
    /** `tag`, `untag`, or both. */
    actions: z.array(Tagging),
    /** How far: `"any"`, the compartments listed, or the tags as a test. */
    within: reach(Listed),
    /** Which tags: `"any"`, or only these. */
    tags: within(z.array(z.string())),
  }),
]);
export type Permission = z.infer<typeof Permission>;
