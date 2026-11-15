// What an icon channel answers.

import { DecodeError, jsonOf, splitTag } from "../../../../wire/decode.js";
import { Error } from "../../../error.js";

/**
 * One of an image's icons, that the image ships none, why it could not
 * be read, or that it is too large to be. Exactly one is sent on an
 * icon's channel of an `images.read` scope, and the finish follows it.
 *
 * - `{ icon }`: the file, byte for byte as the image holds it, at most
 *   1048576 bytes, unread by whoever carried it. Tag `0`.
 * - `"missing"`: the image ships no document, its document names no
 *   such icon, or the file named is not there. Tag `1`.
 * - `{ error }`: the document is not one or is too large to be read,
 *   or the image could not be read. Tag `2`.
 * - `"too_large"`: the icon's file is larger than 1048576 bytes, and
 *   none of it is answered. Tag `3`.
 */
export type Frame = { readonly icon: Uint8Array } | "missing" | { readonly error: Error } | "too_large";

const ICON = 0;
const MISSING = 1;
const ERROR = 2;
const TOO_LARGE = 3;

/**
 * A frame out of its payload: the tag byte, then, for an icon, every
 * byte after it, which is the file; for an error, its JSON. A tag that
 * names no variant is the `unexpected_tag` error.
 */
export function decode(bytes: Uint8Array): Frame {
  const [tag, rest] = splitTag(bytes);
  switch (tag) {
    case ICON:
      return { icon: rest };
    case MISSING:
      return "missing";
    case ERROR:
      return { error: Error.parse(jsonOf(rest)) };
    case TOO_LARGE:
      return "too_large";
    default:
      throw new DecodeError("unexpected_tag", tag);
  }
}
