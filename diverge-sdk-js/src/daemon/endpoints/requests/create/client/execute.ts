// Performing the exchange: one request, a stream to its answer.

import type { Handle } from "../../../../../wire/client/handle.js";
import type { Cancel } from "../../../../../daemon/client/cancel.js";
import { ExecuteStream, executeCancellable } from "../../../../../daemon/client/stream.js";
import * as request from "./request/index.js";
import * as response from "../server/response/index.js";

/**
 * Make the request, and hand back the stream of what becomes of it —
 * `created`, each `delivered`, then `responded` — and the cancel that
 * takes it back from everyone it was sent to.
 */
export function execute(handle: Handle, frame: request.Frame): Promise<[ExecuteStream<response.Frame>, Cancel]> {
  return executeCancellable(handle, frame, request.encode, response.decode);
}
