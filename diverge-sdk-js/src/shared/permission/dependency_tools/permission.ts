// One grant's worth over dependency tools.

import { z } from "zod";
import { reach } from "../reach.js";
import { Tagging } from "../tagging.js";
import { within } from "../within.js";
import { Over } from "./over.js";
import { Listed } from "./listed.js";
import { Within } from "./within.js";

/**
 * What one grant over dependency tools allows, in one of two shapes told apart
 * on the wire by their very form: an object of `Over`
 * actions and how far they reach, by tags; or an object of tagging
 * actions, how far they reach, and which tags. An object that is
 * none of these — an action of one shape among another's, a member
 * missing — does not decode. The shapes are tried in this order, as
 * serde tries them.
 */
export const Permission = z.union([
  /** Over dependency tools that exist: the `Over` actions held, reaching every one, the agent's attached ones, those listed, or only those the tags let through, read as a test. */
  z.strictObject({
    /** The actions. */
    actions: z.array(Over),
    /** How far: `"any"`, `"attached"`, the dependency tools listed, or the tags as a test. See `Within`. */
    within: Within,
  }),
  /** Tagging dependency tools: putting tags on, taking them off, or both, reaching every one or only those the tags let through, and touching any tag or only the ones listed. */
  z.strictObject({
    /** `tag`, `untag`, or both. */
    actions: z.array(Tagging),
    /** How far: `"any"`, the dependency tools listed, or the tags as a test. */
    within: reach(Listed),
    /** Which tags: `"any"`, or only these. */
    tags: within(z.array(z.string())),
  }),
]);
export type Permission = z.infer<typeof Permission>;
