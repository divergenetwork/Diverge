// A create's answer: what becomes of the request.

import { z } from "zod";
import { DecodeError, jsonOf, splitTag } from "../../../../../../wire/decode.js";
import { Error } from "../../../../../../shared/error.js";
import { Created } from "./created.js";
import { Delivered } from "./delivered.js";
import { Responded } from "./responded.js";

/**
 * What becomes of a request, or the one refusal that ends the create.
 * `created`, first; zero or more `delivered`, one for each listener
 * sent it, as each is; then `responded` and the finish — or, where the
 * caller took the request back or went, the finish with no `responded`
 * before it. A create that is refused is exactly one of `forbidden`,
 * `not_found` and `error`, with nothing before it, then the finish.
 */
export const Frame = z.union([
  /** The request exists. Tag `0`. */
  z.strictObject({ created: Created }),
  /** A listener was sent it. Tag `1`. */
  z.strictObject({ delivered: Delivered }),
  /** It was answered. Tag `2`. */
  z.strictObject({ responded: Responded }),
  /** The caller holds no `request` grant, or none reaching an account, a tool or a tag the request names. Tag `3`. */
  z.literal("forbidden"),
  /** An account or a tool the request names is none there is. Tag `4`. */
  z.literal("not_found"),
  /** A failure: a request that names nobody among them. Tag `5`. */
  z.strictObject({ error: Error }),
]);
export type Frame = z.infer<typeof Frame>;

/**
 * A frame out of its payload: the tag byte, then the variant's own
 * JSON, parsed as what the tag promised — nothing for a bare answer.
 * A tag that names no variant is the `unexpected_tag` error.
 */
export function decode(bytes: Uint8Array): Frame {
  const [tag, rest] = splitTag(bytes);
  switch (tag) {
    case 0:
      return { created: Created.parse(jsonOf(rest)) };
    case 1:
      return { delivered: Delivered.parse(jsonOf(rest)) };
    case 2:
      return { responded: Responded.parse(jsonOf(rest)) };
    case 3:
      return "forbidden";
    case 4:
      return "not_found";
    case 5:
      return { error: Error.parse(jsonOf(rest)) };
    default:
      throw new DecodeError("unexpected_tag", tag);
  }
}
