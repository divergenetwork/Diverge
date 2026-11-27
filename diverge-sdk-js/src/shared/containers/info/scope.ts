// How far a dependency tool is shared.

import { z } from "zod";

/**
 * What a dependency tool is one of: one per parent agent, or one for
 * every agent that declares it, whatever template they are made
 * from. A dependency tool's document states one; a standalone tool's
 * has none, and an agent's has none. The scope names the tool — the
 * same declaration under the same scope key is the same tool, joined
 * and not made again. It does not name the tool's databases, which
 * are the same three for every container, but for one thing: a
 * template-scoped tool runs for no one agent, and has no `instance` database.
 * Snake case on the wire: `"instance"`, `"template"`.
 */
export const Scope = z.enum(["instance", "template"]);
export type Scope = z.infer<typeof Scope>;
