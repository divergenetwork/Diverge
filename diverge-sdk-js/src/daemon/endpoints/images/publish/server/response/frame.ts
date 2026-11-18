// A publish's answers.

import { z } from "zod";
import { DecodeError, jsonOf, splitTag } from "../../../../../../wire/decode.js";
import { Error } from "../../../../../../shared/error.js";
import { Added } from "../../../push/server/response/added.js";
import { Blob } from "../../../push/server/response/blob.js";
import { Merged } from "./merged.js";
import { Published } from "./published.js";
import { Uploaded } from "./uploaded.js";

/**
 * What happened, as it happened, or the one answer that ends the
 * publish.
 *
 * 1. In the order it happened: `{ blob }` per blob pulled onto the
 *    provider; `{ added }` per group of platforms that landed there;
 *    at most one `{ merged }`; `{ uploaded }` per blob the registry
 *    took.
 * 2. Then exactly one of the others, and the finish. `{ published }`
 *    is the one that ends it well.
 * 3. `forbidden`, `not_found`, `no_provider`, `registry_unlisted`,
 *    `unsupported` and `platform_repeated` may come with nothing
 *    before them: nothing was asked of any provider.
 * 4. An answer that ends the publish badly takes back nothing told
 *    before it: what was brought in is in the daemon's Store on the
 *    provider still.
 */
export const Frame = z.union([
  /** A blob pulled onto the provider and verified. Tag `0`. */
  z.strictObject({ blob: Blob }),
  /** Some of the platforms taken are now held on the provider. Tag `1`. */
  z.strictObject({ added: Added }),
  /** The parts are one image in the daemon's Store on the provider. Tag `2`. */
  z.strictObject({ merged: Merged }),
  /** A blob the registry took. Tag `3`. */
  z.strictObject({ uploaded: Uploaded }),
  /** The registry holds the image. Tag `4`. */
  z.strictObject({ published: Published }),
  /** The caller's grants do not allow it: no `publish` over images, or none reaching the provider named. Tag `5`. */
  z.literal("forbidden"),
  /** The provider named is none the daemon keeps a record of. Tag `6`. */
  z.literal("not_found"),
  /** No provider was named, and none the caller's grant reaches is connected, keeps a Store and uses the registry named. Tag `7`. */
  z.literal("no_provider"),
  /** The provider named does not use the registry named. Tag `8`. */
  z.literal("registry_unlisted"),
  /** The provider named keeps no Store. Tag `9`. */
  z.literal("unsupported"),
  /** A platform taken has no source, or a digest named is one no source supplies. Tag `10`. */
  z.literal("unavailable"),
  /** Two parts take the same platform, and nothing was asked of any provider. Tag `11`. */
  z.literal("platform_repeated"),
  /** A platform a part names is not one its image has. Tag `12`. */
  z.literal("platform_not_found"),
  /** The provider's Store has no room. Tag `13`. */
  z.literal("insufficient_capacity"),
  /** A source supplied a part and did not prove it holds it. Tag `14`. */
  z.literal("proof_failed"),
  /** A part is not an image the provider takes, or the repository path or a tag is not one. Tag `15`. */
  z.literal("malformed"),
  /** The registry did not take the credential. Tag `16`. */
  z.literal("unauthorized"),
  /** The credential may not write to the repository. Tag `17`. */
  z.literal("denied"),
  /** A failure, in the daemon's words or the provider's. Tag `18`. */
  z.strictObject({ error: Error }),
]);
export type Frame = z.infer<typeof Frame>;

const BARE = {
  5: "forbidden",
  6: "not_found",
  7: "no_provider",
  8: "registry_unlisted",
  9: "unsupported",
  10: "unavailable",
  11: "platform_repeated",
  12: "platform_not_found",
  13: "insufficient_capacity",
  14: "proof_failed",
  15: "malformed",
  16: "unauthorized",
  17: "denied",
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
    case 3:
      return { uploaded: Uploaded.parse(jsonOf(rest)) };
    case 4:
      return { published: Published.parse(jsonOf(rest)) };
    case 18:
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
