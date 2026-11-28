// Every connected standalone tool, the agent's attached connected standalone tools, the
// connected standalone tools listed, or only these.

import { z } from "zod";
import { Tags } from "../tags.js";
import { Listed } from "./listed.js";

/**
 * How far a grant over connected standalone tools reaches: every one there is, the
 * connected standalone tools attached to the agent holding the grant, the
 * connected standalone tools listed, or only those the `Tags` let through. Flat on
 * the wire: the string `"any"`, the string `"attached"`, the object
 * `{"tools":[…]}`, or the tags object as it is. `"attached"` names
 * nothing and reaches the connected standalone tools on the other side of an
 * attachment from the container that holds the grant, judged at every
 * request, whether or not any of them runs. For an agent, those attached to it directly, and those attached to the template it was made from.
 * Held by a tool's container it reaches nothing: a tool is attached
 * to agents, and no connected standalone tool is attached to a tool. See
 * `agents.Within` for the same word the other way. `{"tools":[…]}`
 * reaches exactly the connected standalone tools listed and no other, whatever
 * their tags or their attachments. An empty list reaches nothing.
 */
export const Within = z.union([
  z.literal("any"),
  z.literal("attached"),
  Listed,
  Tags,
]);
export type Within = z.infer<typeof Within>;
