// An edit's answer.

import { z } from "zod";
import { DecodeError, jsonOf, splitTag } from "../../../../../../wire/decode.js";
import { Error } from "../../../../../../shared/error.js";

/**
 * An edit's answer: the description is kept; no such provider, or no
 * image of the digest there; forbidden; no Store; or a failure.
 * Exactly one per scope, before the finish.
 */
export const Frame = z.union([
  /** The description is kept. Tag `0`. */
  z.literal("edited"),
  /** The daemon keeps no record of the provider named, or owns no image by the digest on it. Nothing changed. Tag `1`. */
  z.literal("not_found"),
  /** The caller holds no `edit` grant over images reaching the provider. Tag `2`. */
  z.literal("forbidden"),
  /** The provider named keeps no Store. Tag `3`. */
  z.literal("unsupported"),
  /** A failure, in the daemon's words or the provider's. Tag `4`. */
  z.strictObject({ error: Error }),
]);
export type Frame = z.infer<typeof Frame>;

/**
 * A frame out of its payload: the tag byte, then the error's value for
 * a failure. A tag that names no variant is the `unexpected_tag` error.
 */
export function decode(bytes: Uint8Array): Frame {
  const [tag, rest] = splitTag(bytes);
  switch (tag) {
    case 0:
      return "edited";
    case 1:
      return "not_found";
    case 2:
      return "forbidden";
    case 3:
      return "unsupported";
    case 4:
      return { error: Error.parse(jsonOf(rest)) };
    default:
      throw new DecodeError("unexpected_tag", tag);
  }
}
