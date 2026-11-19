// A listen's answer: a stream kept open.

import { z } from "zod";
import { DecodeError, jsonOf, splitTag } from "../../../../../../wire/decode.js";
import { Error } from "../../../../../../shared/error.js";
import { Asked } from "./asked.js";
import { Withdrawn } from "./withdrawn.js";

/**
 * A request, a withdrawal, or the one refusal that ends the listen.
 * Zero or more `request`s and `withdrawn`s, in the order they happen,
 * until the caller ends it. A listen that is not opened is exactly one
 * of `already_open`, `forbidden` and `error`, with nothing before it,
 * then the finish.
 */
export const Frame = z.union([
  /** A request made of the caller. Tag `0`. */
  z.strictObject({ request: Asked }),
  /** A request is wanted no more. Tag `1`. */
  z.strictObject({ withdrawn: Withdrawn }),
  /** The caller has a listen open already, on this connection or another, and this one is not opened. Tag `2`. */
  z.literal("already_open"),
  /** The account the request is served for is none there is. Tag `3`. */
  z.literal("forbidden"),
  /** A failure. Tag `4`. */
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
      return { request: Asked.parse(jsonOf(rest)) };
    case 1:
      return { withdrawn: Withdrawn.parse(jsonOf(rest)) };
    case 2:
      return "already_open";
    case 3:
      return "forbidden";
    case 4:
      return { error: Error.parse(jsonOf(rest)) };
    default:
      throw new DecodeError("unexpected_tag", tag);
  }
}
