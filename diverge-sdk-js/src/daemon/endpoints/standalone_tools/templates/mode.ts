// How a tool made from a template is kept running.

import { z } from "zod";

/**
 * Lazy or eager: the `mode` a tool template says beside its `type`.
 * A lazy tool runs while it is used; an eager tool is kept while
 * anything holds it. Snake case on the wire: `"lazy"`, `"eager"`;
 * absent is lazy.
 */
export const Mode = z.enum(["lazy", "eager"]);
export type Mode = z.infer<typeof Mode>;
