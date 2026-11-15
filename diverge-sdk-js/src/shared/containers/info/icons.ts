// The icons an image's document names.

import { z } from "zod";

/**
 * Whether `name` may name an icon's file: 1 to 255 bytes, every one
 * an ASCII letter, an ASCII digit, `.`, `-` or `_`; the first not `.`;
 * and not `DIVERGE.md`, which is the document.
 */
export function nameOk(name: string): boolean {
  return /^[A-Za-z0-9_-][A-Za-z0-9._-]{0,254}$/.test(name) && name !== "DIVERGE.md";
}

/** One icon's file name, as `nameOk` passes it. */
const Name = z.string().refine(nameOk);

/**
 * The icons an image ships, as its document's `icons` names them: each
 * member the name of one file in the image's `/.diverge`, beside the
 * document. Every member is optional. What a file holds is its
 * author's word: nothing that carries an icon reads it.
 */
export const Icons = z.strictObject({
  /** The SVG icon's file. Absent, the image ships none. */
  svg: Name.optional(),
  /** The 512 by 512 PNG icon's file. Absent, the image ships none. */
  "512": Name.optional(),
  /** The 64 by 64 PNG icon's file. Absent, the image ships none. */
  "64": Name.optional(),
});
export type Icons = z.infer<typeof Icons>;
