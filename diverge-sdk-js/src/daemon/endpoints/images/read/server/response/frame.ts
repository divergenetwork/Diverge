// An images read's answers on its main stream.

import { z } from "zod";
import { DecodeError, jsonOf, splitTag } from "../../../../../../wire/decode.js";
import { Error } from "../../../../../../shared/error.js";

/**
 * Whether the daemon can supply the image, platform by platform.
 * Exactly one of `{ held }`, `not_found`, `forbidden` and `{ error }`
 * is first. After any but `{ held }` the finish follows at once and no
 * channel is answered.
 *
 * A held scope's main stream is one or more `{ held }`, then exactly
 * one `settled`, then nothing until the finish:
 *
 * 1. Each `{ held }` is what one source added: the platforms it can
 *    supply that the request wants and that no earlier `{ held }` on
 *    the scope carried. With `a`, `b` and `c` wanted, a source holding
 *    `a` and `b` and a later one holding `b` and `c` are `{ held }` of
 *    `a` and `b`, then `{ held }` of `c`.
 * 2. A request that named no platform wants any: its one `{ held }`
 *    carries every platform the first source can supply.
 * 3. `settled` is sent when every platform wanted has been sent, when
 *    every source has answered, or when the caller says `enough`. No
 *    `{ held }` follows it.
 * 4. The scope answers channels from its first `{ held }`, through
 *    the sources kept so far.
 *
 * The finish follows the caller's cancel, or the caller gone.
 */
export const Frame = z.union([
  /** A source can supply these platforms of the image, and the daemon serves the scope through it. Tag `0`, then the platforms as a JSON array. */
  z.strictObject({ held: z.strictObject({ platforms: z.array(z.string()) }) }),
  /** No source the daemon asked can. Tag `1`. */
  z.literal("not_found"),
  /** The account the request is served for holds neither `read` over providers nor `relay` over daemons; nothing was asked. Tag `2`. */
  z.literal("forbidden"),
  /** A failure. See `shared.error.Error` for why it says so little. Tag `3`. */
  z.strictObject({ error: Error }),
  /** The daemon looks no further, and no held follows. Tag `4`. */
  z.literal("settled"),
]);
export type Frame = z.infer<typeof Frame>;

const HELD = 0;
const NOT_FOUND = 1;
const FORBIDDEN = 2;
const ERROR = 3;
const SETTLED = 4;

/**
 * A frame out of its payload: the tag byte, then the platforms of a
 * held, or the error's value. A tag that names no variant is the
 * `unexpected_tag` error.
 */
export function decode(bytes: Uint8Array): Frame {
  const [tag, rest] = splitTag(bytes);
  switch (tag) {
    case HELD:
      return { held: { platforms: z.array(z.string()).parse(jsonOf(rest)) } };
    case SETTLED:
      return "settled";
    case NOT_FOUND:
      return "not_found";
    case FORBIDDEN:
      return "forbidden";
    case ERROR:
      return { error: Error.parse(jsonOf(rest)) };
    default:
      throw new DecodeError("unexpected_tag", tag);
  }
}
