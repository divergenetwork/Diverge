// A request, as a listener is sent it.

import { z } from "zod";
import { ContentBlockSchema } from "@modelcontextprotocol/core";
import { RequestId } from "../../../id.js";

/** One request made of the listener. */
export const Asked = z.object({
  /** What the request is known by: what an answer to it, and its withdrawal, name. */
  id: RequestId,
  /** The request: content blocks, as its creator wrote them. The daemon reads none of it. */
  request: z.array(ContentBlockSchema),
});
export type Asked = z.infer<typeof Asked>;
