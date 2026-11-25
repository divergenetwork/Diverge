// One secret an image's program may take.

import { z } from "zod";

import { Description } from "../../description.js";

/** The most characters a secret's name holds. */
export const SECRET_NAME_CHARACTERS = 75;

/**
 * Whether `name` is a secret's name: one line of plain text, with no
 * control character, of one to `SECRET_NAME_CHARACTERS` characters
 * counted as Unicode scalar values, and trimmed.
 */
export function named(name: string): boolean {
  const characters = [...name].length;
  return characters >= 1 && characters <= SECRET_NAME_CHARACTERS && !/\p{Cc}/u.test(name) && name.trim() === name;
}

/**
 * One secret an image's program may take, as the image's document
 * states it: its name, and what it is, in one line. An image states
 * every secret its program could take, whatever its arguments; which
 * of them a container needs is its template's to say, by name.
 */
export const Secret = z.strictObject({
  name: z.string().refine(named),
  description: Description,
});
export type Secret = z.infer<typeof Secret>;
