// How a tool is kept running.

import { z } from "zod";

/**
 * Lazy or eager: how every tool made from an image is kept running,
 * as the image itself says. A tool's document states one, a
 * standalone tool's and a dependency tool's alike; an agent's has
 * none. A lazy tool runs while anything holds it — an agent it is
 * served to being started, having a message on its way or being
 * active among the holds — and is stopped the caller's idle time after
 * the last hold leaves; an eager tool is held as a lazy tool is, and
 * as well by any agent attached to it, running or not. Every tool an
 * agent is served is deployed when the agent's container starts. Snake case on the wire: `"lazy"`,
 * `"eager"`.
 */
export const Mode = z.enum(["lazy", "eager"]);
export type Mode = z.infer<typeof Mode>;
