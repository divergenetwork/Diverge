// A merge's answers.

import { z } from "zod";
import { DecodeError, jsonOf, splitTag } from "../../../../../../wire/decode.js";
import { Error } from "../../../../../../shared/error.js";
import { Added } from "../../../push/server/response/added.js";
import { Blob } from "../../../push/server/response/blob.js";
import { Merged } from "../../../publish/server/response/merged.js";

/**
 * What happened, as it happened, or the one answer that ends the
 * merge.
 *
 * 1. In the order it happened: `{ blob }` per blob pulled onto the
 *    provider; `{ added }` per group of platforms that landed there.
 * 2. Then exactly one of the others, and the finish. `{ merged }` is
 *    the one that ends it well.
 * 3. `forbidden`, `not_found`, `unsupported` and `platform_repeated`
 *    may come with nothing before them: nothing was asked of the
 *    provider.
 * 4. An answer that ends the merge badly takes back nothing told
 *    before it: what was brought in is in the daemon's Store on the
 *    provider still.
 */
export const Frame = z.union([
  /** A blob pulled onto the provider and verified. Tag `0`. */
  z.strictObject({ blob: Blob }),
  /** Some of the platforms taken are now held on the provider. Tag `1`. */
  z.strictObject({ added: Added }),
  /** The images are one image in the daemon's Store on the provider: the merge's end. Tag `2`. */
  z.strictObject({ merged: Merged }),
  /** The caller's grants do not allow it: no `merge` over images reaching the provider. Tag `3`. */
  z.literal("forbidden"),
  /** The provider named is none the daemon keeps a record of. Tag `4`. */
  z.literal("not_found"),
  /** The provider named keeps no Store. Tag `5`. */
  z.literal("unsupported"),
  /** An image named, or a platform taken of it, is one no source supplies. Tag `6`. */
  z.literal("unavailable"),
  /** Two images take the same platform, and nothing was asked of the provider. Tag `7`. */
  z.literal("platform_repeated"),
  /** A platform an image names is not one it has. Tag `8`. */
  z.literal("platform_not_found"),
  /** The provider's Store has no room. Tag `9`. */
  z.literal("insufficient_capacity"),
  /** A source supplied an image and did not prove it holds it. Tag `10`. */
  z.literal("proof_failed"),
  /** An image is not one the provider takes. Tag `11`. */
  z.literal("malformed"),
  /** A failure, in the daemon's words or the provider's. Tag `12`. */
  z.strictObject({ error: Error }),
]);
export type Frame = z.infer<typeof Frame>;

const BARE = {
  3: "forbidden",
  4: "not_found",
  5: "unsupported",
  6: "unavailable",
  7: "platform_repeated",
  8: "platform_not_found",
  9: "insufficient_capacity",
  10: "proof_failed",
  11: "malformed",
} as const;

/**
 * A frame out of its payload: the tag byte, then the variant's own
 * JSON, parsed as what the tag promised — nothing for a bare answer.
 * A tag that names no variant is the `unexpected_tag` error.
 */
export function decode(bytes: Uint8Array): Frame {
  const [tag, rest] = splitTag(bytes);
  switch (tag) {
    case 0:
      return { blob: Blob.parse(jsonOf(rest)) };
    case 1:
      return { added: Added.parse(jsonOf(rest)) };
    case 2:
      return { merged: Merged.parse(jsonOf(rest)) };
    case 12:
      return { error: Error.parse(jsonOf(rest)) };
    default: {
      const bare = (BARE as Record<number, (typeof BARE)[keyof typeof BARE]>)[tag];
      if (bare === undefined) {
        throw new DecodeError("unexpected_tag", tag);
      }
      return bare;
    }
  }
}
