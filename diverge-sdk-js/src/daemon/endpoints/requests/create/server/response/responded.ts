// The request's answer.

import { z } from "zod";
import { Creator } from "../../../../../creator/creator.js";

/** The one answer the request got. */
export const Responded = z.object({
  /** The answer: any JSON value, as the listener wrote it. The daemon reads none of it. */
  response: z.json(),
  /** Who answered: an account, by its identity, or a tool, as the daemon names a party. One of those the request was delivered to. */
  from: Creator,
});
export type Responded = z.infer<typeof Responded>;
