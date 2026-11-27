// Listing the agent templates, narrowed, and keeping the list.

import { z } from "zod";
import { json, tagged } from "../../../../../../../wire/encode.js";
import { Relay } from "../../../../../../../shared/relay.js";
import { Filter } from "./filter.js";

/**
 * Ask the daemon for the agent templates, narrowed. Everything but `arguments` is optional, and a
 * request with nothing else — `{"arguments":false}` on the wire — is
 * every one the caller's grants reach, each template without its
 * arguments. The members but `arguments` and `count` are the filter,
 * flattened in.
 */
export const Frame = z.object({
  ...Filter.shape,
  /** Whether the list carries each template's arguments. Required, `true` or `false`, and never supposed, so `{}` is no request. `true`, every template the list sends has its `arguments`, and the caller's `arguments` grants reach every one the list shows: where they do not reach one of them the list is answered `forbidden`, and nothing is sent. `false`, no template the list sends has them, and nothing beyond the `list` is asked. */
  arguments: z.boolean(),
  /** How many the list is kept to: the first that many that match. `0` is a list of nothing. Absent, no cap. */
  count: z.number().int().nonnegative().optional(),
  /** Served by another daemon: see `Relay`. Absent, served here. */
  relay: Relay.optional(),
});
export type Frame = z.infer<typeof Frame>;

/** This frame's tag among the scope-opening requests: one byte at the front of the payload. */
const TAG = 11;

/** The payload: the tag, then the frame as JSON, as every request of the daemon's is. */
export function encode(frame: Frame): Uint8Array {
  return tagged(TAG, json(frame));
}
