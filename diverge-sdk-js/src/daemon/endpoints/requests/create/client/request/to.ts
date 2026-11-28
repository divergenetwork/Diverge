// Whom a request is made of.

import { z } from "zod";
import { Identity } from "../../../../../agent_log/identity.js";
import { Agent } from "../../../../../reference/agent.js";
import { Dependency } from "../../../../../reference/dependency.js";
import { Tool } from "../../../../../reference/tool.js";

/**
 * The accounts, agents and tools a request is made of, each named by itself
 * or by a tag. A party is addressed where any member names it. A
 * kind's tags are that kind's: `account_tags` name accounts and
 * nothing else, `agent_tags` agents, and so the two tool kinds. A tag names every record
 * that carries it, those that come to carry it while the request is
 * pending included. What is named and the caller's `request` grant for
 * its kind does not reach is not addressed, and neither is a name that
 * names nothing; neither refuses the create. Every member may be left
 * out. A request that addresses nobody at all is refused.
 */
export const Only = z.strictObject({
  /** Accounts, each by its identity. */
  accounts: z.array(Identity).optional(),
  /** Every account that carries any of these tags. */
  account_tags: z.array(z.string()).optional(),
  /** Agents, each by its name, or its template and index. */
  agents: z.array(Agent).optional(),
  /** Every agent that carries any of these tags. */
  agent_tags: z.array(z.string()).optional(),
  /** Standalone tools, each by its name, or its template and index. */
  standalone_tools: z.array(Tool).optional(),
  /** Every standalone tool that carries any of these tags. */
  standalone_tool_tags: z.array(z.string()).optional(),
  /** Dependency tools, each by its template and its owner. */
  dependency_tools: z.array(Dependency).optional(),
  /** Every dependency tool that carries any of these tags. */
  dependency_tool_tags: z.array(z.string()).optional(),
});
export type Only = z.infer<typeof Only>;

/**
 * Whom a request is made of: the string `"all"` — every account, every
 * agent and every tool the caller's `request` grants reach, and no other — or `{"only": {…}}`, the accounts, agents and tools
 * named.
 */
export const To = z.union([z.literal("all"), z.strictObject({ only: Only })]);
export type To = z.infer<typeof To>;
