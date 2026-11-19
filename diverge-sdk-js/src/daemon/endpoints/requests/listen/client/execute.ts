// Performing the exchange: one request, a stream kept open, and the
// answers given on it.

import type { Handle } from "../../../../../wire/client/handle.js";
import type { Cancel } from "../../../../../daemon/client/cancel.js";
import { ExecuteStream, executeCancellable } from "../../../../../daemon/client/stream.js";
import { unary } from "../../../../../provider/endpoints/containers/client/unary.js";
import type { RequestId } from "../../id.js";
import * as channel_request from "./channel_request/index.js";
import * as request from "./request/index.js";
import * as channel_response from "../server/channel_response/index.js";
import * as response from "../server/response/index.js";

/**
 * What a listener does on its open listen: answer a request, and stop
 * listening. Every answer is a channel of its own, so several may be
 * given at once.
 */
export class Responder {
  readonly #handle: Handle;
  readonly #cancel: Cancel;

  constructor(handle: Handle, cancel: Cancel) {
    this.#handle = handle;
    this.#cancel = cancel;
  }

  /**
   * Answer the request `id` with `response`: whether the daemon took
   * the answer as the request's. `not_delivered` is an answer like the
   * other: the request was answered or withdrawn first, or is none
   * this listener was sent.
   */
  respond(id: RequestId, response: channel_request.Respond["response"]): Promise<channel_response.Frame> {
    return unary<channel_response.Frame, never>(this.#handle, this.#cancel.scope, channel_request.encode({ respond: { id, response } }), (payload) => ({
      item: channel_response.decode(payload),
    }));
  }

  /** Stop listening: the scope finishes, which ends the stream. */
  cancel(): Promise<void> {
    return this.#cancel.cancel();
  }
}

/**
 * Open the listen, and hand back the stream of what the caller is sent
 * — each `request` and each `withdrawn`, or the one refusal — and the
 * `Responder` it answers and ends it with.
 */
export async function execute(handle: Handle, frame: request.Frame): Promise<[ExecuteStream<response.Frame>, Responder]> {
  const [stream, cancel] = await executeCancellable(handle, frame, request.encode, response.decode);
  return [stream, new Responder(handle, cancel)];
}
