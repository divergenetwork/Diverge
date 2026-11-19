// A request, as a listener is sent it.

import { z } from "zod";
import { RequestId } from "../../../id.js";

/** One request made of the listener. */
export const Asked = z.object({
  /** What the request is known by: what an answer to it, and its withdrawal, name. */
  id: RequestId,
  /** The request: any JSON value, as its creator wrote it. The daemon reads none of it. */
  request: z.json(),
});
export type Asked = z.infer<typeof Asked>;
