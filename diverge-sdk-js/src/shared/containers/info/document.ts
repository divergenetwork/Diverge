// An image's DIVERGE.md, parsed.

import { z } from "zod";

import { Description } from "../../description.js";
import { Chat } from "./chat.js";
import { falseOr } from "./false_or.js";
import { StructuredDecisions } from "./structured_decisions.js";

/** What an agent's document and a tool's both have. */
const common = {
  /** The image's own name, as its author states it. */
  name: z.string(),
  /** What the image is, in one line, as its author states it: the one description an image has. */
  description: Description,
  /** The arguments' JSON Schema, a JSON object as the parse requires. */
  schema: z.json(),
  /** Whether the program serves a frontend beneath `/frontend`. Absent, it serves none. */
  frontend: z.boolean().optional(),
  /** The markdown beneath the frontmatter. */
  body: z.string(),
};

/**
 * An agent's image's document: `type` is `agent`. What a tool's has,
 * and how the agent's program chats and makes structured decisions.
 * At least one of the two is an object: an agent does at least one.
 */
export const Agent = z
  .strictObject({
    type: z.literal("agent"),
    ...common,
    /** Whether the program chats, and how: `false`, or a `Chat`. Absent, `false`. */
    chat: falseOr(Chat).optional(),
    /**
     * Whether the program makes structured decisions, and how:
     * `false`, or a `StructuredDecisions`. Absent, `false`.
     */
    structured_decisions: falseOr(StructuredDecisions).optional(),
  })
  .refine((agent) => typeof agent.chat === "object" || typeof agent.structured_decisions === "object");
export type Agent = z.infer<typeof Agent>;

/**
 * A tool's image's document: `type` is `tool`. It has no `chat` and no
 * `structured_decisions`: one that states either is not a document.
 */
export const Tool = z.strictObject({
  type: z.literal("tool"),
  ...common,
});
export type Tool = z.infer<typeof Tool>;

/**
 * An image's `/.diverge/DIVERGE.md`, parsed: an agent's document or a
 * tool's, told apart by `type`. Each is the JSON frontmatter — the
 * type, the image's name, the JSON Schema of the arguments the image's
 * program accepts, whether the program serves a frontend, and, in an
 * agent's alone, how its program chats and
 * makes structured decisions — and the markdown body beneath it. A
 * member that is none of its type's is not allowed.
 */
export const Document = z.union([Agent, Tool]);
export type Document = z.infer<typeof Document>;
