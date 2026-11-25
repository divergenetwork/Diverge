// What an image may call itself.

import { z } from "zod";

/** The most bytes an image's name holds. */
export const NAME_BYTES = 128;

/**
 * Runs of `a` to `z` and `0` to `9`, joined by separators, each `.`,
 * `_`, `__`, or one or more `-`, with no separator first or last.
 */
const RUNS = /^[a-z0-9]+(?:(?:__|[._]|-+)[a-z0-9]+)*$/;

/**
 * An image's name, which is what one path component of an OCI
 * repository name may be: 1 to `NAME_BYTES` bytes, of runs of `a` to
 * `z` and `0` to `9` joined by `.`, `_`, `__` or one or more `-`,
 * neither beginning nor ending with one of those. Every character of
 * such a name is one byte, so its length is its bytes.
 */
export const Name = z.string().refine((name) => name.length <= NAME_BYTES && RUNS.test(name));
export type Name = z.infer<typeof Name>;
