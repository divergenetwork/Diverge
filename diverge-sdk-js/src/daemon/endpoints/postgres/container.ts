// One container, as the database knows it.

import { z } from "zod";
import { Agent } from "../../reference/agent.js";
import { Dependency } from "../../reference/dependency.js";
import { Tool } from "../../reference/tool.js";

/**
 * One container, as the database knows it: an agent or a tool on
 * record, named once and for all, or a dependency tool by its
 * template and whose it is. One object with one member, named for
 * the family: `{"agent":…}`, `{"tool":…}` or `{"dependency":…}`. A
 * container reaches three compartments, and holds a connection
 * through one of them at a time.
 */
export const Container = z.union([
  z.strictObject({ agent: Agent }),
  z.strictObject({ tool: Tool }),
  z.strictObject({ dependency: Dependency }),
]);
export type Container = z.infer<typeof Container>;
