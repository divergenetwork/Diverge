// Whether an answer was taken.

import { z } from "zod";
import { DecodeError, splitTag } from "../../../../../../wire/decode.js";

/**
 * Whether the daemon took a listener's answer as the request's:
 * exactly one of these on the answer's channel, then the finish. One
 * byte on the wire, `0` or `1`.
 *
 * - `delivered`: the answer is the request's. Its creator is sent it,
 *   with who gave it, and the request is over; every other listener it
 *   was sent to is told it is withdrawn, and this one is not.
 * - `not_delivered`: the answer was not taken, and changes nothing:
 *   the request was answered by another listener, was withdrawn, was
 *   never sent to this listener, or is none there is.
 */
export const Frame = z.enum(["delivered", "not_delivered"]);
export type Frame = z.infer<typeof Frame>;

/** A delivery out of its one byte. A byte that is neither is the `unexpected_tag` error. */
export function decode(bytes: Uint8Array): Frame {
  const [tag] = splitTag(bytes);
  switch (tag) {
    case 0:
      return "delivered";
    case 1:
      return "not_delivered";
    default:
      throw new DecodeError("unexpected_tag", tag);
  }
}
