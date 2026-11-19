// Whom a request is made of.

import { z } from "zod";
import { Identity } from "../../../../../agent_log/identity.js";
import { Dependency } from "../../../../../reference/dependency.js";
import { Tool } from "../../../../../reference/tool.js";

/**
 * The accounts and tools a request is made of, each named by itself
 * or by a tag. A party is addressed where any member names it. A
 * kind's tags are that kind's: `account_tags` name accounts and
 * nothing else, and so the two tool kinds. A tag names every record
 * that carries it, those that come to carry it while the request is
 * pending included. Every member may be left out; every member empty
 * names nobody, and the create is refused as an error.
 */
export const Only = z.strictObject({
  /** Accounts, each by its identity. */
  accounts: z.array(Identity).optional(),
  /** Every account that carries any of these tags. */
  account_tags: z.array(z.string()).optional(),
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
 * Whom a request is made of: the string `"all"` — every account and
 * every tool the caller's `request` grants reach, and no other, never
 * refused for reach — or `{"only": {…}}`, the accounts and tools
 * named.
 */
export const To = z.union([z.literal("all"), z.strictObject({ only: Only })]);
export type To = z.infer<typeof To>;
