// The image's document, as the wire carries it parsed.

import { z } from "zod";

import { Description } from "../../description.js";
import { Chat } from "./chat.js";
import { falseOr } from "./false_or.js";
import { Mode } from "./mode.js";
import { Name } from "./name.js";
import { Scope } from "./scope.js";
import { StructuredDecisions } from "./structured_decisions.js";

/**
 * What every document has, whichever of the three it is: the image's
 * own name, what it is in one line, the JSON Schema of its arguments,
 * the memory and the disk a container of it needs, in bytes, and
 * the prose beneath the frontmatter.
 */
const common = {
  name: Name,
  description: Description,
  schema: z.json(),
  memory: z.number().int().nonnegative(),
  disk: z.number().int().nonnegative(),
  body: z.string(),
};

/**
 * What a tool's document has, of either kind, and no agent's: the two
 * doors of its program. `frontend`: whether it serves pages beneath
 * `/frontend`. `api`: whether it serves an API beneath `/api`. Each
 * `true` only where the frontmatter says so; absent, `false`.
 */
const doors = {
  frontend: z.boolean().optional(),
  api: z.boolean().optional(),
};

/**
 * An agent's image's document: `"type":"agent"`. What every document
 * has, and how the program chats and makes structured decisions, at
 * least one of the two an object. It has no `mode` and no `scope`,
 * and neither door: an agent's program serves no frontend and no API,
 * and a document that states `frontend` or `api` is no agent's.
 */
export const Agent = z
  .strictObject({
    type: z.literal("agent"),
    ...common,
    chat: falseOr(Chat).optional(),
    structured_decisions: falseOr(StructuredDecisions).optional(),
  })
  .refine((agent) => typeof agent.chat === "object" || typeof agent.structured_decisions === "object");
export type Agent = z.infer<typeof Agent>;

/**
 * A standalone tool's image's document: `"type":"standalone_tool"`.
 * A standalone tool is made by hand from a tool template, and is no
 * agent's dependency. What every document has, its two doors, and the
 * `mode` its tools are kept by. It has no `scope`.
 */
export const StandaloneTool = z.strictObject({
  type: z.literal("standalone_tool"),
  ...common,
  ...doors,
  mode: Mode,
});
export type StandaloneTool = z.infer<typeof StandaloneTool>;

/**
 * A dependency tool's image's document: `"type":"dependency_tool"`.
 * A dependency tool is declared by an agent's program, and is made by
 * nothing else. What a standalone tool's has, and the `scope` its
 * tools are shared by.
 */
export const DependencyTool = z.strictObject({
  type: z.literal("dependency_tool"),
  ...common,
  ...doors,
  mode: Mode,
  scope: Scope,
});
export type DependencyTool = z.infer<typeof DependencyTool>;

/**
 * `/.diverge/DIVERGE.md`, parsed: an agent's image's document, a
 * standalone tool's or a dependency tool's, told apart by `type`. An
 * image is one of the three and never two.
 */
export const Document = z.union([Agent, StandaloneTool, DependencyTool]);
export type Document = z.infer<typeof Document>;
