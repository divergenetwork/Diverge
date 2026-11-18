// A delete's answer.

import { z } from "zod";
import { DecodeError, jsonOf, splitTag } from "../../../../../../wire/decode.js";
import { Error } from "../../../../../../shared/error.js";

/**
 * A delete's answer: deleted, or why nothing was. After `deleted` the
 * image is gone from the daemon's Store on the provider, or the
 * platforms named of it are; after every other answer nothing was
 * deleted.
 */
export const Frame = z.union([
  /** The image is gone, or the platforms named of it are. Tag `0`. */
  z.literal("deleted"),
  /** A running container runs the image, and nothing was deleted. Tag `1`. */
  z.literal("in_use"),
  /** A platform named is not one the daemon holds of the image there, and nothing was deleted. Tag `2`. */
  z.literal("platform_not_held"),
  /** The provider is none the daemon keeps a record of, or the daemon owns no image of the digest in its Store there. Tag `3`. */
  z.literal("not_found"),
  /** The provider has no Store, and there was nothing to delete. Tag `4`. */
  z.literal("unsupported"),
  /** The account the request is served for holds no grant allowing it: no `delete` over images reaching the provider. Tag `5`. */
  z.literal("forbidden"),
  /** A failure, in the daemon's words or the provider's. Tag `6`. */
  z.strictObject({ error: Error }),
]);
export type Frame = z.infer<typeof Frame>;

const BARE = ["deleted", "in_use", "platform_not_held", "not_found", "unsupported", "forbidden"] as const;
const ERROR = 6;

/**
 * A frame out of its payload: the tag byte, then the error's value
 * for a failure and nothing for a bare answer. A tag that names no
 * variant is the `unexpected_tag` error.
 */
export function decode(bytes: Uint8Array): Frame {
  const [tag, rest] = splitTag(bytes);
  if (tag === ERROR) {
    return { error: Error.parse(jsonOf(rest)) };
  }
  const bare = BARE[tag];
  if (bare === undefined) {
    throw new DecodeError("unexpected_tag", tag);
  }
  return bare;
}
