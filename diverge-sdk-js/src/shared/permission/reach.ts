// Every one, exactly these, or those the tags let through.

import { z } from "zod";
import { Tags } from "./tags.js";

/**
 * How far a grant a tool's program declares reaches, over one kind of
 * thing: every one of the kind, exactly the ones listed, or only
 * those the `Tags` let through. `listed` is the kind's own list, its
 * `Listed`: an object of one member naming the records one by one.
 * Flat on the wire: the string `"any"`; the list as it is,
 * `{"ids":[…]}`; or the tags as they are. One of three and never two:
 * a list has its one member and no other, and the tags refuse it, so
 * a grant that lists records and names tags at once does not decode.
 * A list reaches exactly what it lists, whatever the tags of what it
 * lists; an empty list reaches nothing. "These records, or anything
 * tagged so" is two grants. The generic of the Rust, as a function of
 * the schema it is over.
 */
export function reach<L extends z.ZodType>(listed: L) {
  return z.union([z.literal("any"), listed, Tags]);
}

export type Reach<L> = "any" | L | Tags;
