// An answer to a request.

import { z } from "zod";
import { RequestId } from "../../../id.js";

/** The caller's answer to one request it was sent. */
export const Respond = z.object({
  /** Which request, by the id it came with. */
  id: RequestId,
  /** The answer: any JSON value. The daemon reads none of it, and the creator is sent it as it is. */
  response: z.json(),
});
export type Respond = z.infer<typeof Respond>;
