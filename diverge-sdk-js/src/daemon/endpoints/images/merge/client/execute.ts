// Performing the exchange: one request, and what the merge says.

import type { Handle } from "../../../../../wire/client/handle.js";
import * as stream from "../../../../../daemon/client/stream.js";
import * as request from "./request/index.js";
import * as response from "../server/response/index.js";

/**
 * Send the request and hand back the stream of its answers: what
 * happened as it happened — blobs pulled, platforms added — then the
 * one answer that ends it, then the daemon's finish. There is no
 * cancel: a merge is not stopped from the client's side.
 */
export function execute(handle: Handle, frame: request.Frame): Promise<stream.ExecuteStream<response.Frame>> {
  return stream.execute(handle, frame, request.encode, response.decode);
}
