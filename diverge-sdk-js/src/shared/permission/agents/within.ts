// Every agent, the agents attached to the tool that asks, those never
// active, the agents listed, or only these.

import { z } from "zod";
import { Tags } from "../tags.js";
import { Listed } from "./listed.js";

/**
 * How far a grant over agents reaches: every agent there is, the
 * agents attached to the tool holding the grant, the agents that have
 * never been active, the agents listed, or only those the `Tags` let
 * through. Flat on the wire: the string `"any"`, the string
 * `"attached"`, the string `"never_active"`, the object
 * `{"agents":[…]}`, or the tags object as it is. `"attached"` names
 * nothing and reaches the records on the other side of an attachment
 * from the container that holds the grant: held by an agent's
 * container it reaches nothing, since no agent is attached to an
 * agent, and the three tool kinds have the same word the other way
 * (see `standalone_tools.Within`). It reaches, for a standalone tool, the agents it is
 * attached to directly and every agent of every template it is
 * attached to; for a dependency tool, the agents attached to it — for
 * a tool of the `instance` scope the one agent it was deployed for. `"never_active"` reaches every
 * agent no container has ever been started for, judged at each
 * request, so an agent leaves it the moment it first runs.
 * `{"agents":[…]}` reaches exactly the agents listed, each by its
 * template and index, and no other: what a program whose arguments
 * are a list of agents declares, from those arguments, at each
 * registration. An empty list reaches nothing.
 */
export const Within = z.union([
  z.literal("any"),
  z.literal("attached"),
  z.literal("never_active"),
  Listed,
  Tags,
]);
export type Within = z.infer<typeof Within>;
