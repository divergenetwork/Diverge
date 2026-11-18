// Lazy, or eager.

import { z } from "zod";

/**
 * Lazy or eager: the `mode` a dependency template is tagged with, as
 * a value of its own to narrow a list by. Snake case on the wire, as
 * the tag is: `"lazy"`, `"eager"`.
 */
export const Mode = z.enum(["lazy", "eager"]);
export type Mode = z.infer<typeof Mode>;
