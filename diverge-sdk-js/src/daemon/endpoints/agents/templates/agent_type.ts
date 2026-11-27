// What tells an agent template from a tool template.

import { z } from "zod";

/**
 * `"type":"agent"`, and nothing beside it: what tells an agent
 * template from a tool template with the same image, limits,
 * and arguments. Flat among the template's members.
 */
export const AgentType = z.object({ type: z.literal("agent") });
export type AgentType = z.infer<typeof AgentType>;
