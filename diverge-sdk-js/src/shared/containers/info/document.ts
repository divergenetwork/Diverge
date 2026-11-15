// An image's DIVERGE.md, parsed.

import { z } from "zod";

import { Chat } from "./chat.js";
import { falseOr } from "./false_or.js";
import { Icons } from "./icons.js";
import { StructuredDecisions } from "./structured_decisions.js";

/**
 * An image's `/.diverge/DIVERGE.md`, parsed: the JSON frontmatter — the image's
 * name, the JSON Schema of the arguments the image's program accepts,
 * whether the program serves a frontend, the icons the image ships,
 * and how an agent's program chats and makes structured decisions — and the markdown body
 * beneath it. An agent's document and a tool's are one type: a tool's
 * states neither of the last two, which reads as `false`.
 */
export const Document = z.object({
  /** The image's own name, as its author states it. */
  name: z.string(),
  /** The arguments' JSON Schema, a JSON object as the parse requires. */
  schema: z.json(),
  /** Whether the program serves a frontend beneath `/frontend`. Absent, it serves none. */
  frontend: z.boolean().optional(),
  /** The icons the image ships, each the name of a file beside the document. Absent, none. */
  icons: Icons.optional(),
  /** Whether an agent's program chats, and how: `false`, or a `Chat`. Absent, `false`. */
  chat: falseOr(Chat).optional(),
  /**
   * Whether an agent's program makes structured decisions, and how:
   * `false`, or a `StructuredDecisions`. Absent, `false`.
   */
  structured_decisions: falseOr(StructuredDecisions).optional(),
  /** The markdown beneath the frontmatter. */
  body: z.string(),
});
export type Document = z.infer<typeof Document>;
