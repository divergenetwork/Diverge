// An answer to a request.

import { z } from "zod";
import { ContentBlockSchema } from "@modelcontextprotocol/core";
import { RequestId } from "../../../id.js";

/** The caller's answer to one request it was sent. */
export const Respond = z.object({
  /** Which request, by the id it came with. */
  id: RequestId,
  /** The answer: content blocks, as MCP writes them. The daemon reads none of it, and the creator is sent it as it is. */
  response: z.array(ContentBlockSchema),
});
export type Respond = z.infer<typeof Respond>;
