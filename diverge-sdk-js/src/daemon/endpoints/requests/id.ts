// A request's name.

import { z } from "zod";

/**
 * What a request is known by, from its creation to its end: a string
 * the daemon draws at random when the request is created, and which
 * nothing else is read from. A listener answers a request by it, and
 * is told of a withdrawal by it.
 */
export const RequestId = z.string();
export type RequestId = z.infer<typeof RequestId>;
