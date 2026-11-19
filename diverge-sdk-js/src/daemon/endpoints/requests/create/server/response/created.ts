// The request, made.

import { z } from "zod";
import { RequestId } from "../../../id.js";

/** The request exists: listeners are being sent it. */
export const Created = z.object({
  /** What it is known by, drawn at random by the daemon: what each listener is sent it under. */
  id: RequestId,
});
export type Created = z.infer<typeof Created>;
