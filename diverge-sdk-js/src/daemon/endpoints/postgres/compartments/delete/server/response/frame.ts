// A delete's answer.

import { z } from "zod";
import { DecodeError, jsonOf, splitTag } from "../../../../../../../wire/decode.js";
import { Error } from "../../../../../../../shared/error.js";

/**
 * A delete's answer: the compartment is gone, none is the one named,
 * it is not deletable, it is in use, forbidden, or a failure. Exactly
 * one per scope, before the finish.
 */
export const Frame = z.union([
  /** The compartment is gone, with every table in it. A dependency template's took the template's tools and its body with it. Tag `0`. */
  z.literal("deleted"),
  /** No compartment is the one named; nothing changed. Tag `1`. */
  z.literal("not_found"),
  /** The compartment goes with what it is of and by no delete of its own, or what would keep it is there still; nothing changed. Tag `2`. */
  z.literal("not_deletable"),
  /** A container holds a connection through the compartment, or a dependency tool of its template runs; nothing changed. Tag `3`. */
  z.literal("in_use"),
  /** The account the request is served for holds no grant allowing it; nothing changed. Tag `4`. */
  z.literal("forbidden"),
  /** A failure. See `shared.error.Error` for why it says so little. Tag `5`. */
  z.strictObject({ error: Error }),
]);
export type Frame = z.infer<typeof Frame>;

const DELETED = 0;
const NOT_FOUND = 1;
const NOT_DELETABLE = 2;
const IN_USE = 3;
const FORBIDDEN = 4;
const ERROR = 5;

/**
 * A frame out of its payload: the tag byte, then the variant's own
 * JSON, parsed as what the tag promised — nothing for a bare
 * answer, the error's value for a failure. A tag that names no
 * variant is the `unexpected_tag` error.
 */
export function decode(bytes: Uint8Array): Frame {
  const [tag, rest] = splitTag(bytes);
  switch (tag) {
    case DELETED:
      return "deleted";
    case NOT_FOUND:
      return "not_found";
    case NOT_DELETABLE:
      return "not_deletable";
    case IN_USE:
      return "in_use";
    case FORBIDDEN:
      return "forbidden";
    case ERROR:
      return { error: Error.parse(jsonOf(rest)) };
    default:
      throw new DecodeError("unexpected_tag", tag);
  }
}
